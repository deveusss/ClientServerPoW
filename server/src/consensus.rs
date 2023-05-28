use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

use crate::{middleware::Middleware, server::SERVER_CONFIG};
use rand::Rng;
use sha2::{Digest, Sha256};

const DIFFICULTY_PREFIX: &[u8] = &[0]; // Example difficulty prefix of one leading zero

struct PoWMiddleware {
    difficulty: usize,
    next: Option<Box<dyn Middleware>>,
}

impl PoWMiddleware {
    fn select_quote() -> String {
        let server_config = &*SERVER_CONFIG;
        let quotes = &server_config.quotes;
        let quote_index = rand::thread_rng().gen_range(0..quotes.len());
        quotes[quote_index].clone()
    }

    fn generate_challenge(&self) -> Vec<u8> {
        let mut challenge = vec![0; self.difficulty];
        rand::thread_rng().try_fill(&mut challenge[..]).unwrap();
        challenge
    }

    fn compute_solution(&self, challenge: &[u8]) -> u64 {
        let mut nonce = 0u64;

        loop {
            let solution = nonce.to_be_bytes();
            if self.validate_solution(challenge, &solution) {
                return nonce;
            }

            nonce += 1;
        }
    }

    fn validate_solution(&self, challenge: &[u8], solution: &[u8]) -> bool {
        let prefix_len = DIFFICULTY_PREFIX.len();
        if solution.len() < prefix_len {
            return false;
        }

        let challenge_solution = [challenge, solution].concat();
        let mut hasher = Sha256::new();
        hasher.update(&challenge_solution);
        let result = hasher.finalize();

        for i in 0..prefix_len {
            if result[i] != DIFFICULTY_PREFIX[i] {
                return false;
            }
        }

        true
    }
}

#[async_trait::async_trait]
impl Middleware for PoWMiddleware {
    async fn handle(&self, stream: Arc<Mutex<tokio::net::TcpStream>>, addr: SocketAddr) {
        tracing::info!("Consensus is handling request from client: {}", addr);
        let challenge = self.generate_challenge();
        tracing::info!("Consensus is computing solution");

        let solution = self.compute_solution(&challenge);

        tracing::info!("Consensus is validating solution");
        let is_valid = self.validate_solution(&challenge, &solution.to_be_bytes());
        if !is_valid {
            // PoW solution is invalid, handle accordingly (e.g., reject the request)
            return;
        }

        tracing::info!("Consensus is selecting quate");


        // Send quote as response
        let quote = PoWMiddleware::select_quote();
        let response = format!("Quote of the day: {}", quote);
        stream
            .lock()
            .await
            .write_all(response.as_bytes())
            .await
            .unwrap();

            tracing::info!("Call the next middleware or request handler in the chain");

        // Call the next middleware or request handler in the chain
        if let Some(next_middleware) = &self.next {
            next_middleware.handle(stream, addr).await;
        }
    }
}

pub fn consensus_middleware(
    difficulty: usize,
    next: Option<Box<dyn Middleware>>,
) -> Box<dyn Middleware> {
    let middleware = PoWMiddleware { difficulty, next };
    Box::new(middleware)
}
