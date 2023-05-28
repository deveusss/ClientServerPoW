use tracing::{Level};
use tracing_subscriber::{fmt};

mod consensus;
mod middleware;
mod server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    setup_logging();

    let bind_address = format!(
        "{}:{}",
        server::SERVER_CONFIG.ip,
        server::SERVER_CONFIG.port
    );
    server::start_server(&bind_address).await
}

fn setup_logging() {
    // Set the desired log level and configure the subscriber
    let log_level = Level::INFO; // Adjust the log level as needed

    let fmt_subscriber = fmt::SubscriberBuilder::default().finish();

    // Initialize the subscriber
    tracing::subscriber::set_global_default(fmt_subscriber).expect("Failed to set logger");

    tracing::info!("Logger initialized with level: {:?}", log_level);
    tracing::info!("Starting server...");
}
