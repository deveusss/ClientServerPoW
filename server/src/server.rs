use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::consensus::consensus_middleware;
use crate::middleware::{
    middleware_ddos_protection, middleware_logging, middleware_request_timeout, Middleware,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub ip: String,
    pub port: u16,
    pub max_connections: usize,
    pub ddos_threshold: usize,
    pub challenge_length: usize,
    pub quotes: Vec<String>,
}

pub static SERVER_CONFIG: Lazy<ServerConfig> =
    Lazy::new(|| load_server_config().expect("Failed to load server configuration"));

fn load_server_config() -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let config_str = match fs::read_to_string("server_config.json") {
        Ok(content) => content,
        Err(_) => include_str!("server_config.json").to_owned(),
    };

    let server_config: ServerConfig = serde_json::from_str(&config_str)?;
    Ok(server_config)
}

pub const REQUEST_TIMEOUT_SECONDS: u64 = 5;

async fn handle_connection(
    stream: Arc<Mutex<TcpStream>>,
    middleware_chain: Arc<Mutex<Vec<Box<dyn Middleware>>>>,
) {

    let addr = (*stream.lock().await).peer_addr().unwrap();
    tracing::info!("New connection from: {}", addr);

    let middleware_chain = middleware_chain.lock().await;

    for middleware in middleware_chain.iter() {
        middleware.handle(Arc::clone(&stream), addr).await;
    }
}

pub async fn start_server(bind_address: &str) -> std::io::Result<()> {

    let listener = TcpListener::bind(bind_address).await?;

    let request_counts = Arc::new(Mutex::new(Default::default()));
    let server_config = {
        ServerConfig {
            ddos_threshold: 5,
            ..(*SERVER_CONFIG).clone()
        }
    };


    let middleware_chain: Arc<Mutex<Vec<Box<dyn Middleware>>>> = Arc::new(Mutex::new(vec![
        middleware_ddos_protection(
            Arc::clone(&request_counts),
            server_config.ddos_threshold,
            Some(middleware_request_timeout(
                Duration::from_secs(REQUEST_TIMEOUT_SECONDS),
                Some(middleware_logging(None)),
            )),
        ),
        consensus_middleware(server_config.challenge_length, None),
        // Add your PoW middleware here
    ]));

    tracing::info!("Server started at: {}", bind_address);

    loop {
        let (stream, _) = listener.accept().await?;
        tracing::info!("Handle connection");

        tokio::spawn(handle_connection(
            Arc::new(Mutex::new(stream)),
            Arc::clone(&middleware_chain),
        ));
    }
}
