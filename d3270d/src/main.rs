use std::ffi::OsString;
use std::future::Future;
use std::pin::Pin;
use std::process::Stdio;
use std::str::FromStr;
use std::task::{ready, Context, Poll};

use anyhow::anyhow;
use futures::future::select_all;
use futures::FutureExt;
use tokio::task::JoinHandle;
use tracing::{error, info};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;

use d3270_common::b3270::operation::Action;

pub mod arbiter;
pub mod gen_connection;
pub mod tcp_server;

struct TaggedJoinHandle {
    handle: JoinHandle<anyhow::Error>,
    tag: &'static str,
}

impl Future for TaggedJoinHandle {
    type Output = (&'static str, anyhow::Error);

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let err = match ready!(self.handle.poll_unpin(cx)) {
            Ok(err) => err,
            Err(err) => err.into(),
        };
        Poll::Ready((self.tag, err))
    }
}

trait JoinHandleTagExt {
    fn tagged(self, tag: &'static str) -> TaggedJoinHandle;
}

impl JoinHandleTagExt for JoinHandle<anyhow::Error> {
    fn tagged(self, tag: &'static str) -> TaggedJoinHandle {
        TaggedJoinHandle { handle: self, tag }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::EnvFilter::from_default_env())
        .with(
            tracing_subscriber::fmt::layer()
                .with_span_events(FmtSpan::ACTIVE)
        ).init();

    info!("Test");

    let mut subprocess_args = vec![
        OsString::from_str("-json").unwrap(),
        OsString::from_str("-utf8").unwrap(),
    ];
    let mut args_iter = std::env::args_os().peekable();
    let mut connect_str = None;
    let mut tcp_listen = None;

    args_iter.next(); // skip program name.

    while let Some(arg) = args_iter.next() {
        // we default to one of the ignored args
        match arg.to_str().unwrap_or("-json") {
            "-json" | "-xml" | "-indent" | "--" | "-scriptportonce" | "-nowrapperdoc"
            | "-socket" | "-v" | "--version" => {}
            "-scriptport" | "-httpd" => {
                args_iter.next();
            }
            "-connect" => {
                connect_str = args_iter
                    .next()
                    .ok_or_else(|| anyhow!("Arg required for -connect"))
                    .and_then(|arg| {
                        arg.into_string()
                            .map_err(|_| anyhow!("Invalid connect string"))
                    })
                    .map(Some)?;
            }
            "-tcp-listen" => {
                tcp_listen = args_iter
                    .next()
                    .ok_or_else(|| anyhow!("Arg required for -tcp-listen"))?
                    .into_string()
                    .map_err(|_| anyhow!("Failed to parse tcp-listen address"))?
                    .parse()
                    .map(Some)
                    .map_err(|_| anyhow!("Invalid listen address"))?;
            }
            "-e" => {
                'skip: while let Some(arg) = args_iter.peek() {
                    if arg.to_str().unwrap_or("").starts_with("-") {
                        break 'skip;
                    }
                    args_iter.next();
                }
            }
            _ => subprocess_args.push(arg),
        }
    }

    let connect_str = connect_str.ok_or_else(|| anyhow!("No connect string given"))?;

    info!(args=?subprocess_args, "Starting b3270");
    let subproc = tokio::process::Command::new("b3270")
        .args(&subprocess_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut handles: Vec<TaggedJoinHandle> = vec![];

    let (arbiter, arbiter_req) = arbiter::B3270::spawn(
        subproc,
        &[Action {
            action: "Connect".to_owned(),
            args: vec![connect_str],
        }],
    );
    handles.push(arbiter.tagged("arbiter"));
    if let Some(addr) = tcp_listen {
        let tcp_listener = tcp_server::listener_proc(addr, arbiter_req.clone()).await?;
        handles.push(tcp_listener.tagged("tcp_listener"));
    }
    let ((source, error), _, _) = select_all(handles).await;
    error!(source, %error, "A core task failed");

    Ok(())
}
