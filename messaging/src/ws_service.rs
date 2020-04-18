use crate::headpose_service::head_pose_api_client::HeadPoseApiClient;
use crate::headpose_service::Frame;

use async_tungstenite::{accept_async, tungstenite::Error, tokio::TokioAdapter};
use futures_io::{AsyncRead, AsyncWrite};
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::net::SocketAddr;
use std::sync::{ Arc, atomic::{AtomicI32, Ordering}};
use tonic::Request;
use tonic::transport::channel::Channel;
use tokio::net::{TcpListener, TcpStream};
use tokio::task;
use tungstenite::{Message, Result};
use image::jpeg::JpegDecoder;

#[derive(Clone)]
pub struct WebSocketService {
    headpose_client: HeadPoseApiClient<Channel>,
}

impl WebSocketService {
    pub fn new(headpose_client: HeadPoseApiClient<Channel>) -> Self { 
        Self {
            headpose_client,
        } 
    }

    async fn accept_connection(&self, peer: SocketAddr, stream: TcpStream) {
        if let Err(e) = self.handle_connection(peer, Box::pin(TokioAdapter(stream))).await {
            match e {
                Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
                err => log::error!("Error accepting connection: {}", err),
            }
        }
    }

    async fn handle_connection<T: AsyncRead + AsyncWrite + Unpin>(&self, peer: SocketAddr, stream: T) -> Result<()> {
        let mut ws_stream = accept_async(stream).await.expect("Failed to accept");
        log::info!("accepted new Socket connection: {}", peer);
        let face_missing = Arc::new(AtomicI32::new(0));

        while let Some(msg) = ws_stream.next().await {
            let msg = msg?;
            let frame_bytes = match msg {
               Message::Text(ref data) => {
                   if let Ok(url) = data_url::DataUrl::process(data) {
                       match url.decode_to_vec() {
                           Err(e) => {
                               log::error!("could not decode base64 message {:?}", e);
                               None
                           },
                           Ok((bytes, _)) => {
                               Some(bytes)
                           }
                       } 
                   } else {
                       log::error!("invalid data url");
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
                    let request_frame = Frame {
                        frame_identifier: 0,
                        height: 0,
                        width: 0,
                        frame_data: bytes,
                    };

                    let response = self.headpose_client.clone().get_pose(Request::new(request_frame)).await;
                    match response {
                        Err(e) => {
                            log::error!("error: {:?}", e);
                        },
                        Ok(data) => {
                            log::info!("received: {:?}", data);
                            let data = data.get_ref();
                            let current_value = if data.pose.len() == 0 {
                                face_missing.fetch_add(1, Ordering::SeqCst)
                            } else {
                                let current_value = face_missing.fetch_sub(1, Ordering::SeqCst);
                                if current_value <= 0 {
                                    face_missing.store(0, Ordering::SeqCst);
                                    0
                                } else {
                                    current_value
                                }
                            };

                            if current_value > 20 {
                                face_missing.store(20, Ordering::SeqCst);
                            }


                            if current_value > 5 {
                                let message = format!("{{\"events\": [ {{\"name\": \"face_not_detected\", \"p\": {}}} ] }}", current_value as f32 / 20_f32);
                                log::info!("sending: {}", message);
                                ws_stream.send(Message::text(message)).await?;
                            }
                        }
                    }

                } else {
                    log::error!("invalid jpeg image");
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
                log::error!("peer did not have a peer address");
            }

        }
    }
}
