use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpResponse, HttpServer, Responder};
use openssl::ssl::SslAcceptorBuilder;
use serde::Deserialize;

use backend_server::kv_client::KvClient;
use tonic::{transport::Channel, Code};
use tracing::{error, info, warn};
use tracing_actix_web::TracingLogger;

use crate::backend_server::{GetValueRequest, InsertValueRequest};

pub mod backend_server {
    tonic::include_proto!("kv");
}

#[derive(Deserialize, Debug)]
struct KV {
    key: String,
    value: String,
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Hello")
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

pub async fn run(
    listener: TcpListener,
    kv_client: KvClient<Channel>,
    ssl: Option<SslAcceptorBuilder>,
) -> Result<Server, std::io::Error> {
    let kv_client = web::Data::new(kv_client);

    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/{key}", web::get().to(get_value))
            .route("/", web::post().to(insert_value))
            .app_data(kv_client.clone())
    });

    if let Some(builder) = ssl {
        server = server.listen_openssl(listener, builder)?;
    } else {
        server = server.listen(listener)?;
    }

    let server = server.run();

    Ok(server)
}
