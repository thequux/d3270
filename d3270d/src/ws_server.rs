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

use std::net::SocketAddr;
use anyhow::anyhow;
use tide::prelude::*;
use tide::{Request, Response};
use tide_websockets::{WebSocketConnection, self as ws};
use tokio::select;
use tokio::task::JoinHandle;
use crate::arbiter::ArbiterHandleRequester;
use crate::gen_connection::GenConnection;
use futures::stream::StreamExt;
use tracing::{info, warn};
use d3270_common::b3270::Indication;
use rust_embed::{EmbeddedFile, RustEmbed};
use tide::http::{mime, StatusCode};


#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/../js3270/dist/"]
struct Asset;

async fn static_file(req: Request<ArbiterHandleRequester>) -> tide::Result {
    let mut path = req.param("path").unwrap_or("index.html");
    if path == "" {
        path = "index.html"
    }
    let content_type = if let Some((_, ext)) = path.rsplit_once(".") {
        match ext {
            "svg" => mime::SVG,
            "jpg" | "jpeg" => mime::JPEG,
            "png" => mime::PNG,
            "css" => mime::CSS,
            "js" => mime::JAVASCRIPT,
            "html" | "htm" => mime::HTML,
            _ => mime::BYTE_STREAM,
        }
    } else {
        mime::BYTE_STREAM
    };

    if let Some(file) = Asset::get(path) as Option<EmbeddedFile> {
        Ok(Response::builder(StatusCode::Ok)
            .content_type(content_type)
            .body(file.data.as_ref())
            .build())
    } else {
        Ok(Response::builder(StatusCode::NotFound)
            .content_type(mime::PLAIN)
            .body("Dude, where's your file?")
            .build())
    }
}

pub async fn start_ws_server(socket: SocketAddr, handle_requester: ArbiterHandleRequester, ) -> anyhow::Result<JoinHandle<anyhow::Error>> {
    let mut app = tide::Server::with_state(handle_requester);
    app.with(tide_tracing::TraceMiddleware::new());
    app.at("/api/ws").get(tide_websockets::WebSocket::new(handle_websocket));
    app.at("/*path").get(static_file);
    app.at("/").get(static_file);

    let mut listener = app.bind(socket.clone()).await?;
    Ok(tokio::task::spawn(async move {
        info!(address=%socket, "Starting HTTP server");
        listener.accept().await
            .map(|()| anyhow!("HTTP server returned early"))
            .unwrap_or_else(Into::into)
    }))
}

async fn handle_websocket(req: Request<ArbiterHandleRequester>, mut ws: WebSocketConnection) -> tide::Result<()> {
    info!("Handling websocket");
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