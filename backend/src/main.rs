use tracing_subscriber::{EnvFilter, FmtSubscriber};

use config::get_configuration;

mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    FmtSubscriber::builder().with_env_filter(filter).init();

    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!("{}:{}", configuration.host, configuration.application_port);

    backend::run(address).await?;

    Ok(())
}
