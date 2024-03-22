use tracing_subscriber::{EnvFilter, FmtSubscriber};

use config::get_configuration;

use tonic::transport::Identity;

mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    FmtSubscriber::builder().with_env_filter(filter).init();

    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!("{}:{}", configuration.host, configuration.application_port);

    let cert = std::fs::read_to_string("cert2.pem").expect("cert2.pem should exist.");
    let key = std::fs::read_to_string("key2.pem").expect("key2.pem should exist.");

    let identity = Identity::from_pem(cert, key);

    backend::run(address, Some(identity)).await?;

    Ok(())
}
