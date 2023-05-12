/*************************************************************************
 * D3270 - Detachable 3270 interface                                      *
 * Copyright (C) 2023  Daniel Hirsch                                      *
 *                                                                        *
 * This program is free software: you can redistribute it and/or modify   *
 * it under the terms of the GNU General Public License as published by   *
 * the Free Software Foundation, either version 3 of the License, or      *
 * (at your option) any later version.                                    *
 *                                                                        *
 * This program is distributed in the hope that it will be useful,        *
 * but WITHOUT ANY WARRANTY; without even the implied warranty of         *
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the          *
 * GNU General Public License for more details.                           *
 *                                                                        *
 * You should have received a copy of the GNU General Public License      *
 * along with this program.  If not, see <https://www.gnu.org/licenses/>. *
 *************************************************************************/

use crate::arbiter::{ArbiterHandle, ArbiterHandleRequester};
use d3270_common::b3270::indication::RunResult;
use d3270_common::b3270::operation::Run;
use d3270_common::b3270::{Indication, Operation};
use futures::stream::FuturesUnordered;
use futures::stream::StreamExt;
use futures::FutureExt;
use std::future::{poll_fn, Future};
use std::pin::Pin;
use std::task::{ready, Context, Poll};
use tokio::sync::oneshot;
use tracing::warn;

pub struct GenConnection {
    handle: ArbiterHandle,
    waiting_actions: FuturesUnordered<ReplaceTag>,
}

struct ReplaceTag {
    tag: Option<String>,
    rcvr: oneshot::Receiver<RunResult>,
}

impl Future for ReplaceTag {
    type Output = Option<Indication>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(ready!(self.rcvr.poll_unpin(cx)).ok().map(|run_res| {
            Indication::RunResult(RunResult {
                r_tag: self.tag.take(),
                ..run_res
            })
        }))
    }
}

impl GenConnection {
    pub async fn new(ahr: ArbiterHandleRequester) -> anyhow::Result<Self> {
        let handle = ahr.connect().await?;
        Ok(Self {
            handle,
            waiting_actions: FuturesUnordered::new(),
        })
    }

    pub async fn handle_client_line(&mut self, line: String) -> anyhow::Result<()> {
        let op = serde_json::from_str(&line)?;
        match op {
            Operation::Run(Run { actions, r_tag, .. }) => {
                let rcvr = self.handle.send_actions(actions).await?;
                self.waiting_actions.push(ReplaceTag { tag: r_tag, rcvr });
            }
            _ => warn!(json = line, "Unsupported operation from client"),
        }
        Ok(())
    }

    pub fn poll_indication(&mut self, cx: &mut Context) -> Poll<Option<Indication>> {
        let mut any_can_continue = false;

        match self.waiting_actions.poll_next_unpin(cx) {
            Poll::Ready(Some(ind)) => {
                return Poll::Ready(ind);
            }
            Poll::Ready(None) => {}
            Poll::Pending => any_can_continue = true,
        }

        match self.handle.poll_next_unpin(cx) {
            Poll::Ready(Some(ind)) => {
                return Poll::Ready(Some(ind));
            }
            Poll::Ready(None) => {}
            Poll::Pending => any_can_continue = true,
        }

        if any_can_continue {
            Poll::Pending
        } else {
            Poll::Ready(None)
        }
    }

    pub async fn next_indication(&mut self) -> Option<Indication> {
        poll_fn(|cx| self.poll_indication(cx)).await
    }
}
