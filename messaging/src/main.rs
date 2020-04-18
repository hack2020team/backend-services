#[macro_use] extern crate failure;

mod ws_service;
mod headpose_service;

use env_logger::Env;
use failure::Error;
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
