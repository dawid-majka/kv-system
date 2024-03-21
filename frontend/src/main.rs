use std::{io::ErrorKind, net::TcpListener, time::Duration};

use frontend::{backend_server::kv_client::KvClient, run};
use tokio::time::sleep;
use tonic::transport::Channel;
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::config::get_configuration;

mod config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    FmtSubscriber::builder().with_env_filter(filter).init();

    let configuration = get_configuration().expect("Failed to read configuration.");

    let listener = TcpListener::bind(format!(
        "{}:{}",
        configuration.frontend.host, configuration.frontend.application_port
    ))
    .expect("Should bind to '127.0.0.1:8000'");

    let address = &format!(
        "http://{}:{}",
        configuration.backend.host, configuration.backend.application_port
    );

    info!("Connecting to grpc server with address: {}", address);
    let kv_client = try_connect(address)
        .await
        .map_err(|e| std::io::Error::new(ErrorKind::ConnectionRefused, e.to_string()))?;

    run(listener, kv_client).await?.await
}

async fn try_connect(url: &str) -> Result<KvClient<Channel>, tonic::transport::Error> {
    let mut attempt = 0;
    let max_attempts = 5;
    let base_delay = 500;

    loop {
        match KvClient::connect(url.to_owned()).await {
            Ok(client) => {
                info!("Connection estabilished.");
                return Ok(client);
            }
            Err(_) if attempt < max_attempts => {
                let delay = Duration::from_millis(base_delay * 2_u64.pow(attempt));
                warn!("Failed to connect to backend. Retry in {:?}...", delay);

                sleep(delay).await;
                attempt += 1;
            }
            Err(e) => {
                error!("Failed to connect to grpc server. Error: {:?}", e);
                return Err(e);
            }
        }
    }
}
