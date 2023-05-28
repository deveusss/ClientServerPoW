use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;
use tokio::time;
use std::time::Duration;

#[async_trait::async_trait]
pub trait Middleware: Send + Sync {
    async fn handle(&self, stream: Arc<Mutex<tokio::net::TcpStream>>, addr: SocketAddr);
}

struct DDoSMiddleware {
    request_counts: Arc<Mutex<HashMap<String, usize>>>,
    ddos_threshold: usize,
    next: Option<Box<dyn Middleware>>,
}

#[async_trait::async_trait]
impl Middleware for DDoSMiddleware {
    async fn handle(&self, stream: Arc<Mutex<tokio::net::TcpStream>>, addr: SocketAddr) {
        let client_ip = {
            let stream = stream.lock().await;
            stream
                .peer_addr()
                .map(|addr| addr.ip().to_string())
                .unwrap_or_else(|err| err.to_string())
        };

        let mut request_counts = self.request_counts.lock().await;
        let request_count = request_counts.entry(client_ip.clone()).or_insert(0);
        *request_count += 1;

        if *request_count > self.ddos_threshold {
            tracing::error!("DDOS protection triggered for client: {}", client_ip);

            {
                let mut stream = stream.lock().await;
                let writer = &mut *stream;
                writer
                    .write_all(b"Too many requests\n")
                    .await
                    .expect("Failed to send error message");
                writer
                    .flush()
                    .await
                    .expect("Failed to flush stream");
            }

            return;
        }

        // Call the next middleware or request handler in the chain
        if let Some(next_middleware) = &self.next {
            next_middleware.handle(stream, addr).await;
        }
    }
}

pub fn middleware_ddos_protection(
    request_counts: Arc<Mutex<HashMap<String, usize>>>,
    ddos_threshold: usize,
    next: Option<Box<dyn Middleware>>,
) -> Box<dyn Middleware> {
    Box::new(DDoSMiddleware {
        request_counts,
        ddos_threshold,
        next,
    })
}

struct RequestTimeoutMiddleware {
    timeout: Duration,
    next: Option<Box<dyn Middleware>>,
}

#[async_trait::async_trait]
impl Middleware for RequestTimeoutMiddleware {
    async fn handle(&self, stream: Arc<Mutex<tokio::net::TcpStream>>, addr: SocketAddr) {
        let timeout = self.timeout;
        let timeout_fut = time::sleep(timeout);

        tokio::select! {
            _ = timeout_fut => {
                tracing::info!("Request timed out for client: {}", addr);

                {
                    let mut stream = stream.lock().await;
                    let writer = &mut *stream;
                    writer
                        .write_all(b"Request timed out\n")
                        .await
                        .expect("Failed to send timeout message");
                    writer
                        .flush()
                        .await
                        .expect("Failed to flush stream");
                }
            }
            _ = async {
                let timeout_fut = time::sleep(timeout);
                let _ = timeout_fut.await;
            } => {}
        }

        // Call the next middleware or request handler in the chain
        if let Some(next_middleware) = &self.next {
            next_middleware.handle(stream, addr).await;
        }
    }
}

pub fn middleware_request_timeout(timeout: Duration, next: Option<Box<dyn Middleware>>) -> Box<dyn Middleware> {
    Box::new(RequestTimeoutMiddleware {
        timeout,
        next,
    })
}


struct LoggingMiddleware {
    next: Option<Box<dyn Middleware>>,
}

#[async_trait::async_trait]
impl Middleware for LoggingMiddleware {
    async fn handle(&self, stream: Arc<Mutex<tokio::net::TcpStream>>, addr: SocketAddr) {
        tracing::info!("Handling request from client: {}", addr);

        // Call the next middleware or request handler in the chain
        if let Some(next_middleware) = &self.next {
            next_middleware.handle(stream, addr).await;
        }
    }
}

pub fn middleware_logging(next: Option<Box<dyn Middleware>>) -> Box<dyn Middleware> {
    Box::new(LoggingMiddleware { next })
}
