use backend_server::{
    kv_server::Kv, kv_server::KvServer, GetValueRequest, GetValueResponse, InsertValueRequest,
    InsertValueResponse,
};
use frontend::backend_server::kv_client::KvClient;
use reqwest::StatusCode;
use serde_json::json;
use std::{net::TcpListener, time::Duration};
use tokio::time::sleep;
use tonic::{transport::Server, Request, Response, Status};

pub mod backend_server {
    tonic::include_proto!("kv");
}

#[derive(Default)]
pub struct BackendService {}

#[tonic::async_trait]
impl Kv for BackendService {
    #[tracing::instrument(skip(self))]
    async fn insert_value(
        &self,
        _: Request<InsertValueRequest>,
    ) -> Result<Response<InsertValueResponse>, Status> {
        return Ok(Response::new(InsertValueResponse { success: true }));
    }

    async fn get_value(
        &self,
        request: Request<GetValueRequest>,
    ) -> Result<Response<GetValueResponse>, Status> {
        let request = request.into_inner();

        match request.key.as_str() {
            "key1" => {
                return Ok(Response::new(GetValueResponse {
                    value: "value1".to_string(),
                }));
            }
            _ => {
                let status =
                    Status::not_found(format!("Value for key: {} not found.", &request.key));
                return Err(status);
            }
        }
    }
}

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", address))
        .send()
        .await
        .expect("Request should be sent.");

    assert!(response.status().is_success());
}

#[tokio::test]
async fn insert_value_should_return_200() {
    let address = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .post(&format!("{}/", address))
        .json(&json!({"key": "key1", "value": "value1"}))
        .send()
        .await
        .expect("Request should be sent.");

    assert!(response.status().is_success());
}

#[tokio::test]
async fn insert_value_with_empty_key_should_return_400() {
    let address = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .post(&format!("{}/", address))
        .json(&json!({"key": "", "value": "value1"}))
        .send()
        .await
        .expect("Request should be sent.");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_value_should_return_ok() {
    let address = spawn_app().await;

    let client = reqwest::Client::new();

    let key = "key1";
    let value = "value1";

    let response = client
        .get(&format!("{}/{}", address, key))
        .send()
        .await
        .expect("Request should be sent.");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().await.unwrap(), value);
}

#[tokio::test]
async fn get_value_with_invalid_key_should_return_404() {
    let address = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/invalid_key", address))
        .send()
        .await
        .expect("Request should be sent.");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

async fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Should bind to random port.");
    let port = listener.local_addr().unwrap().port();

    let addr = "[::1]:50052".parse().unwrap();
    let svc = BackendService::default();

    tokio::spawn(
        Server::builder()
            .add_service(KvServer::new(svc))
            .serve(addr),
    );

    sleep(Duration::from_secs(1)).await;

    let kv_client = KvClient::connect("http://[::1]:50052").await.unwrap();

    let server = frontend::run(listener, kv_client, None)
        .await
        .expect("Frontend server should be initialized.");

    tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
