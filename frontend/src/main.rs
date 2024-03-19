use std::sync::Arc;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;

use backend_server::kv_client::KvClient;
use tokio::sync::Mutex;
use tonic::transport::Channel;

use crate::backend_server::{GetValueRequest, InsertValueRequest};

pub mod backend_server {
    tonic::include_proto!("kv");
}

#[derive(Deserialize, Debug)]
struct KV {
    key: String,
    value: String,
}

struct AppState {
    kv_client: Arc<Mutex<KvClient<Channel>>>,
}

impl AppState {
    fn new(kv_client: KvClient<Channel>) -> Self {
        AppState {
            kv_client: Arc::new(Mutex::new(kv_client)),
        }
    }
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Hello")
}

async fn get_value(path: web::Path<String>, app_state: web::Data<AppState>) -> impl Responder {
    let key = path.into_inner();
    println!("Got GET request with key: {}", key);

    let mut kv_client = app_state.kv_client.lock().await;

    let request = GetValueRequest { key };

    let response = kv_client
        .get_value(request)
        .await
        .expect("Should send get request to backend server");

    let value = response.into_inner().value;

    println!("Value returned from backend server: {}", &value);

    HttpResponse::Ok().body(value)
}

async fn insert_value(json_data: web::Json<KV>, app_state: web::Data<AppState>) -> impl Responder {
    println!("Got POST request with KV: {:?}", json_data);

    let mut kv_client = app_state.kv_client.lock().await;

    let KV { key, value } = json_data.into_inner();

    let request = InsertValueRequest { key, value };

    let response = kv_client
        .insert_value(request)
        .await
        .expect("Should send insert request to backend server");

    let value = response.into_inner().success;

    println!("Message returned from backend server: {}", &value);

    HttpResponse::Ok().body("Success")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let kv_client = KvClient::connect("http://[::1]:50051")
        .await
        .expect("Connection with Backend service should be estabilished.");

    let app_state = web::Data::new(AppState::new(kv_client));

    HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/{key}", web::get().to(get_value))
            .route("/", web::post().to(insert_value))
            .app_data(app_state.clone())
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
