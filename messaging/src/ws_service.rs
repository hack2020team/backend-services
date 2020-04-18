use async_tungstenite::{accept_async, tungstenite::Error, tokio::TokioAdapter};
use futures_io::{AsyncRead, AsyncWrite};
use futures_util::stream::StreamExt;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::task;
use tungstenite::{Message, Result};
use image::jpeg::JpegDecoder;

#[derive(Clone)]
pub struct WebSocketService {
    
}

impl WebSocketService {
    pub fn new() -> Self { Self {} }

    async fn accept_connection(&self, peer: SocketAddr, stream: TcpStream) {
        if let Err(e) = self.handle_connection(peer, Box::pin(TokioAdapter(stream))).await {
            match e {
                Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
                err => error!("Error accepting connection: {}", err),
            }
        }
    }

    async fn handle_connection<T: AsyncRead + AsyncWrite + Unpin>(&self, peer: SocketAddr, stream: T) -> Result<()> {
        let mut ws_stream = accept_async(stream).await.expect("Failed to accept");
        info!("accepted new Socket connection: {}", peer);

        while let Some(msg) = ws_stream.next().await {
            let msg = msg?;
            let frame_bytes = match msg {
               Message::Text(ref data) => {
                   if let Ok(url) = data_url::DataUrl::process(data) {
                       match url.decode_to_vec() {
                           Err(e) => {
                               error!("could not decode base64 message {:?}", e);
                               None
                           },
                           Ok((bytes, _)) => {
                               Some(bytes)
                           }
                       } 
                   } else {
                       error!("invalid data url");
                       None
                   }

               },
               Message::Binary(data) => {
                   Some(data)
               },
               _ => {
                   None
               }
            };

            if let Some(bytes) = frame_bytes {
                if let Ok(_) = JpegDecoder::new(bytes.as_slice()) {
                    info!("valid jpeg image");
                    // WOOOOOHHHHH assign an ID and ship it off to the ML workers :ok:
                } else {
                    error!("invalid jpeg image");
                }
            }
        }

        Ok(())
    }

    pub async fn run(self, mut listener: TcpListener) {
        while let Ok((stream, _)) = listener.accept().await {
            if let Ok(peer) = stream.peer_addr() {
                let this = self.clone();
                task::spawn(async move {
                    this.accept_connection(peer, stream).await;
                });
            } else {
                error!("peer did not have a peer address");
            }

        }
    }
}
