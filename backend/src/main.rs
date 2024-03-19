use backend_server::kv_server::{Kv, KvServer};
use backend_server::{GetValueRequest, GetValueResponse, InsertValueRequest, InsertValueResponse};

use tonic::{transport::Server, Request, Response, Status};

pub mod backend_server {
    tonic::include_proto!("kv");
}

#[derive(Default)]
pub struct BackendService {}

#[tonic::async_trait]
impl Kv for BackendService {
    async fn insert_value(
        &self,
        request: Request<InsertValueRequest>,
    ) -> Result<Response<InsertValueResponse>, Status> {
        println!("Got insert request: {:?}", request);

        let reply = InsertValueResponse { success: true };

        Ok(Response::new(reply))
    }

    async fn get_value(
        &self,
        request: Request<GetValueRequest>,
    ) -> Result<Response<GetValueResponse>, Status> {
        println!("Got get request: {:?}", request);

        let reply = GetValueResponse {
            value: "value".into(),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let backend_service = BackendService::default();

    Server::builder()
        .add_service(KvServer::new(backend_service))
        .serve(addr)
        .await?;

    Ok(())
}
