use tonic::transport::Server;
use tracing::{error, info};
use uuid::Uuid;

use backend_server::kv_server::{Kv, KvServer};
use backend_server::{GetValueRequest, GetValueResponse, InsertValueRequest, InsertValueResponse};

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

pub mod backend_server {
    tonic::include_proto!("kv");
}

#[derive(Default, Debug)]
pub struct BackendService {
    database: Arc<Mutex<HashMap<String, String>>>,
}

impl BackendService {
    pub fn new() -> Self {
        BackendService {
            database: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[tonic::async_trait]
impl Kv for BackendService {
    #[tracing::instrument(skip(self))]
    async fn insert_value(
        &self,
        request: Request<InsertValueRequest>,
    ) -> Result<Response<InsertValueResponse>, Status> {
        let request = request.into_inner();

        let mut database = self.database.lock().await;

        info!("Inserting data to database.");

        database.insert(request.key, request.value);

        info!("Data inserted succesfully");

        let reply = InsertValueResponse { success: true };

        Ok(Response::new(reply))
    }

    #[tracing::instrument(skip(self))]
    async fn get_value(
        &self,
        request: Request<GetValueRequest>,
    ) -> Result<Response<GetValueResponse>, Status> {
        let request = request.into_inner();

        let database = self.database.lock().await;

        info!("Retrieving data from database.");

        match database.get(&request.key) {
            Some(value) => {
                info!("Value from db: {:?}", value);
                let reply = GetValueResponse {
                    value: value.to_string(),
                };

                return Ok(Response::new(reply));
            }
            None => {
                error!("Value for key: {} not found.", &request.key);
                let status =
                    Status::not_found(format!("Value for key: {} not found.", &request.key));

                return Err(status);
            }
        };
    }
}

pub async fn run(address: String) -> Result<(), Box<dyn std::error::Error>> {
    let address = address.parse()?;
    let backend_service = BackendService::new();

    tracing::info!(message = "Starting server.", %address);

    Server::builder()
        .trace_fn(|_| {
            let request_id = Uuid::new_v4();
            tracing::info_span!("Request span", %request_id)
        })
        .add_service(KvServer::new(backend_service))
        .serve(address)
        .await?;

    Ok(())
}
