use std::time::Duration;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;

use backend_server::kv_client::KvClient;
use tokio::time::sleep;
use tonic::{transport::Channel, Code};
use tracing::{error, info, warn};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::backend_server::{GetValueRequest, InsertValueRequest};

pub mod backend_server {
    tonic::include_proto!("kv");
}

#[derive(Deserialize, Debug)]
struct KV {
    key: String,
    value: String,
}

#[tracing::instrument(
    skip(path, kv_client)
    fields(
        key = %path.as_str()
    )
)]
async fn get_value(
    path: web::Path<String>,
    kv_client: web::Data<KvClient<Channel>>,
) -> impl Responder {
    let key = path.into_inner();

    let mut kv_client = kv_client.get_ref().clone();

    let request = GetValueRequest { key };

    info!("Sending request to grpc server: {:?}", &request);

    let response = kv_client.get_value(request).await;

    match response {
        Ok(response) => {
            let value = response.into_inner().value;

            info!("Value returned from backend server: {}", &value);

            HttpResponse::Ok().body(value)
        }
        Err(status) => {
            error!("Error returned from backend server: {:?}", &status);

            if status.code() == Code::NotFound {
                HttpResponse::NotFound().finish()
            } else {
                HttpResponse::InternalServerError().finish()
            }
        }
    }
}

#[tracing::instrument(skip(kv_client))]
async fn insert_value(
    json_data: web::Json<KV>,
    kv_client: web::Data<KvClient<Channel>>,
) -> impl Responder {
    let mut kv_client = kv_client.get_ref().clone();

    let KV { key, value } = json_data.into_inner();

    if key.trim().is_empty() {
        warn!("Validation failed: key is empty.");
        return HttpResponse::BadRequest().body("'key' field can't be empty.");
    }

    if value.trim().is_empty() {
        warn!("Validation failed: value is empty.");
        return HttpResponse::BadRequest().body("'value' field can't be empty.");
    }

    let request = InsertValueRequest { key, value };

    info!("Sending request to grpc server: {:?}", &request);
    let response = kv_client.insert_value(request).await;

    match response {
        Ok(response) => {
            let value = response.into_inner().success;
            info!("Value returned from backend server: {}", &value);

            HttpResponse::Ok().finish()
        }
        Err(status) => {
            error!("Error returned from backend server: {:?}", &status);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    FmtSubscriber::builder().with_env_filter(filter).init();

    let address = "http://[::1]:50051";

    info!("Connecting to grpc server with address: {}", address);

    let kv_client = match try_connect(address).await {
        Ok(client) => {
            info!("Connection estabilished.");
            client
        }
        Err(e) => {
            error!("Failed to connect to grpc server. Error: {:?}", e);
            error!("Shutting down.");
            std::process::exit(1);
        }
    };

    let kv_client = web::Data::new(kv_client);

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/{key}", web::get().to(get_value))
            .route("/", web::post().to(insert_value))
            .app_data(kv_client.clone())
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}

async fn try_connect(url: &str) -> Result<KvClient<Channel>, tonic::transport::Error> {
    let mut attempt = 0;
    let max_attempts = 5;
    let base_delay = 500;

    loop {
        match KvClient::connect(url.to_owned()).await {
            Ok(client) => return Ok(client),
            Err(_) if attempt < max_attempts => {
                let delay = Duration::from_millis(base_delay * 2_u64.pow(attempt));
                warn!("Failed to connect to backend. Retry in {:?}...", delay);

                sleep(delay).await;
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }
}
