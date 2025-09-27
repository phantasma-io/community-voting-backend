use config::ApiConfig;
use tokio::signal::unix::SignalKind;

pub mod config;
pub mod server;
pub mod error;
pub mod doc;
pub mod data;

#[tokio::main]
async fn main() {
    let config = init_prepare().await;

    let join_handle = tokio::spawn(async move {
        tokio::select! {
            _ = handle_shutdown_recv() => {
                log::warn!("Shutting down...")
            }
            _ = server::run_server(config) => {
                log::warn!("Shutting down because of server...")
            }
        };
    });

    join_handle.await.unwrap();
}

pub async fn init_prepare() -> ApiConfig {
    let config = ApiConfig::parse_from_file();

    init_env();

    config
}

pub fn init_env() {
    dotenvy::dotenv().ok();
    env_logger::try_init().ok();
}

async fn handle_shutdown_recv() {
    let mut sigterm = tokio::signal::unix::signal(SignalKind::terminate()).unwrap();
    let mut sigint = tokio::signal::unix::signal(SignalKind::interrupt()).unwrap();

    tokio::select! {
        _ = sigterm.recv() => {
            log::warn!("Receive SIGTERM")
        }
        _ = sigint.recv() => {
            log::warn!("Receive SIGINT")
        }
    }
}
