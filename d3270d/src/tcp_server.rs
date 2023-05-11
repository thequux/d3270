use std::net::SocketAddr;

use anyhow::bail;
use futures::never::Never;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::task::JoinHandle;
use tracing::{error, info, info_span, Instrument};

use crate::arbiter::ArbiterHandleRequester;
use crate::gen_connection::GenConnection;

pub async fn listener_proc(
    socket: SocketAddr,
    handle_requester: ArbiterHandleRequester,
) -> anyhow::Result<JoinHandle<anyhow::Error>> {
    let listener = tokio::net::TcpListener::bind(socket.clone()).await?;
    let span = info_span!(target: "connection-handling", "tcp_listener", addr=%socket);
    Ok(tokio::spawn(
        async move {
            let error = listener_task(listener, handle_requester).await.unwrap_err();
            error!(%error, "TCP listener failed to accept");
            error
        }
        .instrument(span),
    ))
}

async fn listener_task(
    listener: TcpListener,
    handle_requester: ArbiterHandleRequester,
) -> anyhow::Result<Never> {
    loop {
        let (conn, client_addr) = listener.accept().await?;
        let handle_requester = handle_requester.clone();
        let conn_span =
            info_span!(target: "connection-handling", "tcp_accept", client=%client_addr);
        tokio::spawn(
            async move {
                info!("Accepted connection");
                if let Err(error) = handle_tcp_connection(conn, handle_requester).await {
                    error!(%error, "Connection handler failed");
                } else {
                    info!("Connection closed");
                }
            }
            .instrument(conn_span),
        );
    }
}

async fn handle_tcp_connection(
    mut conn: TcpStream,
    handle_requester: ArbiterHandleRequester,
) -> anyhow::Result<()> {
    let (stream_rd, mut stream_wr) = conn.split();
    let mut stream_rd = BufReader::new(stream_rd).lines();

    let mut conn = GenConnection::new(handle_requester).await?;

    loop {
        select! {
            line = stream_rd.next_line() => match line? {
                Some(line) => conn.handle_client_line(line).await?,
                None => bail!("Connection closed"),
            },
            ind = conn.next_indication() => match ind {
                None => bail!("Arbiter lost"),
                Some(ind) => {
                    let mut ind = serde_json::to_vec(&ind)?;
                    ind.push(b'\n');
                    stream_wr.write_all(ind.as_slice()).await?;
                }
            },
        }
    }
}
