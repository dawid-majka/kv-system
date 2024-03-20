use std::collections::HashMap;
use std::sync::Arc;

use backend_server::kv_server::{Kv, KvServer};
use backend_server::{GetValueRequest, GetValueResponse, InsertValueRequest, InsertValueResponse};

use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};

pub mod backend_server {
    tonic::include_proto!("kv");
}

#[derive(Default)]
pub struct BackendService {
    database: Arc<Mutex<HashMap<String, String>>>,
}

impl BackendService {
    fn new() -> Self {
        BackendService {
            database: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[tonic::async_trait]
impl Kv for BackendService {
    async fn insert_value(
        &self,
        request: Request<InsertValueRequest>,
    ) -> Result<Response<InsertValueResponse>, Status> {
        println!("Got insert request: {:?}", request);

        let request = request.into_inner();

        let mut database = self.database.lock().await;

        database.insert(request.key, request.value);

        let reply = InsertValueResponse { success: true };

        Ok(Response::new(reply))
    }

    async fn get_value(
        &self,
        request: Request<GetValueRequest>,
    ) -> Result<Response<GetValueResponse>, Status> {
        println!("Got get request: {:?}", request);

        let request = request.into_inner();

        let database = self.database.lock().await;

        match database.get(&request.key) {
            Some(value) => {
                let reply = GetValueResponse {
                    value: value.to_string(),
                };

                return Ok(Response::new(reply));
            }
            None => {
                let status =
                    Status::not_found(format!("Value for key: {} not found.", &request.key));

                return Err(status);
            }
        };
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let backend_service = BackendService::new();

    Server::builder()
        .add_service(KvServer::new(backend_service))
        .serve(addr)
        .await?;

    Ok(())
}
