use std::{io::ErrorKind, time::Duration};

use frontend::backend_server::kv_client::KvClient;
use tokio::time::sleep;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint};
use tracing::{error, info, warn};

pub async fn get_client(address: String) -> Result<KvClient<Channel>, std::io::Error> {
    let pem = std::fs::read_to_string("cert2.pem").expect("cert2.pem should exist.");
    let ca = Certificate::from_pem(pem);

    let tls = ClientTlsConfig::new()
        .ca_certificate(ca)
        .domain_name("localhost");

    info!("Connecting to grpc server with address: {}", address);

    let channel = Channel::from_shared(address)
        .unwrap()
        .tls_config(tls)
        .unwrap();

    let kv_client = try_connect(channel)
        .await
        .map_err(|e| std::io::Error::new(ErrorKind::ConnectionRefused, e.to_string()))?;

    Ok(kv_client)
}

async fn try_connect(channel: Endpoint) -> Result<KvClient<Channel>, tonic::transport::Error> {
    let mut attempt = 0;
    let max_attempts = 5;
    let base_delay = 500;

    loop {
        match channel.connect().await {
            Ok(channel) => {
                info!("Connection estabilished.");
                return Ok(KvClient::new(channel));
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
