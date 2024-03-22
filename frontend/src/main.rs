use std::net::TcpListener;

use client::get_client;
use frontend::run;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::config::get_configuration;

mod client;
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

    let address = format!(
        "https://{}:{}",
        configuration.backend.host, configuration.backend.application_port
    );

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key1.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert1.pem").unwrap();

    let kv_client = get_client(address).await?;

    run(listener, kv_client, Some(builder)).await?.await
}
