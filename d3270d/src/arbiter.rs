use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use anyhow::anyhow;
use base64::engine::general_purpose::STANDARD as B64_STANDARD;
use base64::Engine;
use bytes::Buf;
use futures::{FutureExt, Stream, StreamExt, TryFutureExt};
use rand::RngCore;
use tokio::io::{AsyncBufReadExt, AsyncWrite, BufReader, Lines};
use tokio::process::{Child, ChildStdout};
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream};
use tracing::{error, info, info_span, instrument, trace, warn, Instrument};

use d3270_common::b3270::indication::RunResult;
use d3270_common::b3270::operation::Action;
use d3270_common::b3270::{operation, Indication, Operation};
use d3270_common::tracker::{Disposition, Tracker};

enum B3270Request {
    Action(Vec<Action>, oneshot::Sender<RunResult>),
    Resync(oneshot::Sender<(Vec<Indication>, broadcast::Receiver<Indication>)>),
}

enum HandleReceiveState {
    Steady(BroadcastStream<Indication>),
    Wait(oneshot::Receiver<(Vec<Indication>, broadcast::Receiver<Indication>)>),
    Resume(
        std::vec::IntoIter<Indication>,
        broadcast::Receiver<Indication>,
    ),
    TryRestart(
        Pin<Box<dyn Future<Output = Result<(), ()>> + Send + Sync>>,
        oneshot::Receiver<(Vec<Indication>, broadcast::Receiver<Indication>)>,
    ),
}
pub struct ArbiterHandle {
    sender: mpsc::Sender<B3270Request>,
    receiver: Option<HandleReceiveState>,
}

impl ArbiterHandle {
    pub async fn send_action(
        &self,
        action: Action,
    ) -> anyhow::Result<oneshot::Receiver<RunResult>> {
        self.send_actions(vec![action]).await
    }

    #[instrument(skip(self))]
    pub async fn send_actions(
        &self,
        actions: Vec<Action>,
    ) -> anyhow::Result<oneshot::Receiver<RunResult>> {
        let (os_snd, os_rcv) = oneshot::channel();
        self.sender
            .send(B3270Request::Action(actions, os_snd))
            .await
            .map_err(|_| anyhow!("Failed to send action to arbiter"))?;
        Ok(os_rcv)
    }
}

impl Stream for ArbiterHandle {
    type Item = Indication;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match self.receiver.take() {
                Some(HandleReceiveState::TryRestart(mut fut, receiver)) => {
                    if fut.poll_unpin(cx).is_pending() {
                        self.receiver = Some(HandleReceiveState::TryRestart(fut, receiver));
                        return Poll::Pending;
                    }
                    // The option is only there to solve a lifetime issue, so this unwrap is safe
                    self.receiver = Some(HandleReceiveState::Wait(receiver));
                }
                Some(HandleReceiveState::Wait(mut rcvr)) => {
                    match rcvr.poll_unpin(cx) {
                        Poll::Ready(Ok((inds, rcvr))) => {
                            // reverse the indicators so that they can be popped.
                            self.receiver =
                                Some(HandleReceiveState::Resume(inds.into_iter(), rcvr));
                        }
                        Poll::Ready(Err(error)) => {
                            warn!(%error, "unable to reconnect to b3270 server");
                            return Poll::Ready(None);
                        }
                        Poll::Pending => {
                            self.receiver = Some(HandleReceiveState::Wait(rcvr));
                            return Poll::Pending;
                        }
                    }
                }
                Some(HandleReceiveState::Resume(mut inds, rcvr)) => match inds.next() {
                    Some(next) => {
                        self.receiver = Some(HandleReceiveState::Resume(inds, rcvr));
                        return Poll::Ready(Some(next));
                    }
                    None => {
                        self.receiver =
                            Some(HandleReceiveState::Steady(BroadcastStream::new(rcvr)));
                    }
                },
                Some(HandleReceiveState::Steady(mut rcvr)) => match rcvr.poll_next_unpin(cx) {
                    Poll::Ready(Some(Ok(msg))) => {
                        self.receiver = Some(HandleReceiveState::Steady(rcvr));
                        return Poll::Ready(Some(msg));
                    }
                    Poll::Ready(Some(Err(BroadcastStreamRecvError::Lagged(n)))) => {
                        info!(
                            dropped = n,
                            "Dropped messages from b3270 server; starting resync"
                        );
                        let (os_snd, os_rcv) = oneshot::channel();
                        let fut = self
                            .sender
                            .clone()
                            .reserve_owned()
                            .map_ok(move |permit| {
                                permit.send(B3270Request::Resync(os_snd));
                            })
                            .map_err(|_| ());

                        self.receiver = Some(HandleReceiveState::TryRestart(Box::pin(fut), os_rcv));
                    }
                    Poll::Ready(None) => {
                        warn!("Failed to receive from b3270 server");
                        return Poll::Ready(None);
                    }
                    Poll::Pending => {
                        self.receiver = Some(HandleReceiveState::Steady(rcvr));
                        return Poll::Pending;
                    }
                },

                None => {
                    return Poll::Ready(None);
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct ArbiterHandleRequester(mpsc::Sender<B3270Request>);

impl ArbiterHandleRequester {
    #[instrument(skip(self))]
    pub async fn connect(&self) -> anyhow::Result<ArbiterHandle> {
        let (conn_send, conn_rcv) = oneshot::channel();
        self.0
            .send(B3270Request::Resync(conn_send))
            .await
            .map_err(|_| anyhow!("Failed to send request to arbiter"))?;

        let (indications, rcvr) = conn_rcv.await?;
        Ok(ArbiterHandle {
            sender: self.0.clone(),
            receiver: Some(HandleReceiveState::Resume(indications.into_iter(), rcvr)),
        })
    }
}

pub struct B3270 {
    tracker: Tracker,
    child: Child,
    comm: mpsc::Receiver<B3270Request>,
    ind_chan: broadcast::Sender<Indication>,
    child_reader: Lines<BufReader<ChildStdout>>,

    write_buf: VecDeque<u8>,
    action_response_map: HashMap<String, oneshot::Sender<RunResult>>,
}

impl B3270 {
    pub fn spawn(
        mut child: Child,
        initial_actions: &[Action],
    ) -> (
        tokio::task::JoinHandle<anyhow::Error>,
        ArbiterHandleRequester,
    ) {
        let (subproc_snd, subproc_rcv) = mpsc::channel(10);
        let child_reader = child
            .stdout
            .take()
            .expect("Should always be given a child that has stdout captured");
        let child_reader = BufReader::new(child_reader).lines();
        // A single connect can result in a flurry of messages, so we need a big buffer
        let (ind_chan, _) = broadcast::channel(100);

        let mut write_buf = VecDeque::new();
        // Queue any initial actions.
        for action in initial_actions {
            serde_json::to_writer(&mut write_buf, action).unwrap();
            write_buf.push_back(b'\n');
        }

        let proc = B3270 {
            child,
            child_reader,
            tracker: Tracker::default(),
            comm: subproc_rcv,
            ind_chan,
            write_buf,
            action_response_map: Default::default(),
        };
        (
            tokio::task::spawn(proc.instrument(info_span!("arbiter"))),
            ArbiterHandleRequester(subproc_snd),
        )
    }
}

impl Future for B3270 {
    type Output = anyhow::Error;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // try to read data from the child
        let mut indications = vec![];
        // handle new indications first, so that new subscribers get the results in the sync state.
        while let Poll::Ready(buf) = Pin::new(&mut self.child_reader).poll_next_line(cx) {
            match buf {
                Ok(Some(line)) => match serde_json::from_str(&line) {
                    Ok(ind) => {
                        trace!(json = line, "Received indication");
                        indications.push(ind)
                    }
                    Err(error) => {
                        warn!(%error, msg=line, "Failed to parse indication");
                    }
                },
                // EOF on stdin; this is a big problem
                Ok(None) => return Poll::Ready(anyhow!("Child exited unexpectedly")),
                Err(err) => return Poll::Ready(anyhow!(err).context("Failed to read from child")),
            }
        }

        for mut ind in indications {
            match self.tracker.handle_indication(&mut ind) {
                Disposition::Broadcast => {
                    // It's OK to drop these, as anybody who cares will resync
                    self.ind_chan.send(ind).ok();
                }
                Disposition::Drop => {
                    // do nothing
                }
                Disposition::Direct(dst) => {
                    // TODO: handle this once we have a map of destinations.
                    if let Indication::RunResult(run_res) = ind {
                        if let Some(dest) = self.action_response_map.remove(&dst) {
                            // If this fails, whoever sent the request must not have cared.
                            dest.send(run_res).ok();
                        }
                    }
                }
            }
        }
        // check if the server has exited; if so, no sense looking at new connections
        match self.child.try_wait() {
            Ok(Some(status)) => {
                info!(%status, "b3270 process exited");
                return Poll::Ready(anyhow!("b3270 process exited"));
            }
            Ok(None) => {}
            Err(error) => {
                warn!(%error, "Failed to check status of b3270");
                // TODO: should we end now?
            }
        }

        // Only now do we handle connection requests. This way new connections
        // cache the sync state in case we have multiple requests for it at once
        let mut sync_state = None;
        while let Poll::Ready(cmd) = self.comm.poll_recv(cx) {
            match cmd {
                None => {}
                Some(B3270Request::Resync(sender)) => {
                    if sync_state.is_none() {
                        sync_state = Some(self.tracker.get_init_indication());
                    }
                    // it's OK for this to fail; we just don't get a new client
                    sender
                        .send((sync_state.clone().unwrap(), self.ind_chan.subscribe()))
                        .ok();
                }
                Some(B3270Request::Action(actions, response_chan)) => {
                    let tag = 'find_tag: loop {
                        let tag = rand::thread_rng().next_u64().to_le_bytes();
                        let tag = B64_STANDARD.encode(tag);
                        if !self.action_response_map.contains_key(&tag) {
                            break 'find_tag tag;
                        }
                    };
                    let op = Operation::Run(operation::Run {
                        r_tag: Some(tag.clone()),
                        type_: Some("keymap".to_owned()),
                        actions,
                    });
                    match serde_json::to_string(&op) {
                        Ok(op_str) => {
                            trace!(json = op_str, "Sending operation");
                            self.write_buf.extend(op_str.bytes());
                            self.write_buf.push_back(b'\n');
                            self.action_response_map.insert(tag, response_chan);
                        }
                        Err(error) => error!(?op, %error, "Failed to serialize op"),
                    }
                }
            }
        }

        // Now, check if there's anything to be written
        'write: while !self.write_buf.is_empty() {
            let myself = &mut *self;
            let chunk = myself.write_buf.chunk();
            let stdin = Pin::new(
                myself
                    .child
                    .stdin
                    .as_mut()
                    .expect("Should always have child stdin"),
            );
            match stdin.poll_write(cx, chunk) {
                Poll::Pending | Poll::Ready(Ok(0)) => {
                    break 'write;
                }
                Poll::Ready(Ok(n)) => {
                    myself.write_buf.advance(n);
                }
                Poll::Ready(Err(error)) => {
                    warn!(%error, "Failed to write to b3270");
                }
            }
        }
        // We only complete when the child dies, which we catch above
        Poll::Pending
    }
}
