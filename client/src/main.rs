use std::env;
use std::io::{self, Read};
use tokio::net::TcpStream;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let server_address = match args.len() {
        2 => &args[1],
        _ => {
            eprintln!("Usage: ./client <server-address>");
            return Ok(());
        }
    };

    println!("Connecting to server {}", server_address);

    let mut retry_count = 0;
    const MAX_RETRIES: u32 = 3;
    const RETRY_DELAY: u64 = 2000;

    loop {
        match TcpStream::connect(server_address).await {
            Ok(mut stream) => {
                println!("Connected to server");

                let request = "Hello, server!";
                if let Err(err) = stream.write_all(request.as_bytes()).await {
                    if err.kind() == io::ErrorKind::ConnectionReset {
                        eprintln!("Connection reset by peer. Retrying...");
                        continue;
                    } else {
                        return Err(err);
                    }
                }

                let mut buffer = Vec::new();
                if let Err(err) = stream.read_to_end(&mut buffer).await {
                    if err.kind() == io::ErrorKind::ConnectionReset {
                        eprintln!("Connection reset by peer. Retrying...");
                        continue;
                    } else {
                        return Err(err);
                    }
                }

                let response = String::from_utf8_lossy(&buffer);
                println!("Server response: {}", response);

                break;
            }
            Err(err) => {
                retry_count += 1;
                if retry_count > MAX_RETRIES {
                    eprintln!("Failed to connect to server: {}", err);
                    return Err(err);
                }

                eprintln!(
                    "Failed to connect to server (retry {}/{}). Retrying in {} ms...",
                    retry_count,
                    MAX_RETRIES,
                    RETRY_DELAY
                );

                thread::sleep(Duration::from_millis(RETRY_DELAY));
            }
        }
    }

    Ok(())
}
