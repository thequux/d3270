use std::net::SocketAddr;
use anyhow::anyhow;
use tide::prelude::*;
use tide::Request;
use tide_websockets::{WebSocketConnection, self as ws};
use tokio::select;
use tokio::task::JoinHandle;
use crate::arbiter::ArbiterHandleRequester;
use crate::gen_connection::GenConnection;
use futures::stream::StreamExt;
use tracing::{info, warn};
use d3270_common::b3270::Indication;

pub async fn start_ws_server(socket: SocketAddr, handle_requester: ArbiterHandleRequester, ) -> anyhow::Result<JoinHandle<anyhow::Error>> {
    let mut app = tide::Server::with_state(handle_requester);
    app.with(tide_tracing::TraceMiddleware::new());
    app.at("/api/ws").get(tide_websockets::WebSocket::new(handle_websocket));

    let mut listener = app.bind(socket.clone()).await?;
    Ok(tokio::task::spawn(async move {
        info!(%socket, "Starting HTTP server");
        listener.accept().await
            .map(|()| anyhow!("HTTP server returned early"))
            .unwrap_or_else(Into::into)
    }))
}

async fn handle_websocket(req: Request<ArbiterHandleRequester>, mut ws: WebSocketConnection) -> tide::Result<()> {
    let mut arbiter = GenConnection::new(req.state().clone()).await?;

    // TODO: do authenticatey things

    'main: loop {
        select! {
            msg = ws.next() => {
                let msg: ws::Message = if let Some(msg) = msg { msg? } else { break 'main; };
                match msg {
                    ws::Message::Text(text) => arbiter.handle_client_line(text).await?,
                    ws::Message::Binary(_) => warn!("Unexpected binary message from client"),
                    ws::Message::Ping(data) => ws.send(ws::Message::Pong(data)).await?,
                    ws::Message::Close(_) => break 'main,
                    _ => (),
                }
            },
            msg = arbiter.next_indication() => {
                let msg: Indication = if let Some(msg) = msg { msg } else { break 'main; };
                ws.send_json(&msg).await?;
            }
        }
    }

    Ok(())
}