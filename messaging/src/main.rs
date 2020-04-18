#[macro_use] extern crate failure;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;

mod ws_service;

use env_logger::Env;
use failure::Error;
use futures::future;
use std::collections::HashMap;
use tonic::transport::{Server as TonicServer};
use tokio::net::TcpListener;
use ws_service::WebSocketService;


#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::from_env(Env::default().default_filter_or("info")).init();
    let ws_addr = "0.0.0.0:9999";

    let ws_listener = TcpListener::bind(ws_addr).await?;
    let ws_service = WebSocketService::new();
    ws_service.run(ws_listener).await;
    Ok(())
}
