use backend::{
    backend_server::{
        kv_client::KvClient, kv_server::KvServer, GetValueRequest, InsertValueRequest,
    },
    BackendService,
};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{transport::Server, Code};

#[tokio::test]
async fn get_value_request_should_return_not_found_when_invalid_key() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let service = BackendService::new();

    tokio::spawn(async move {
        Server::builder()
            .add_service(KvServer::new(service))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .unwrap();
    });

    let mut client = KvClient::connect(format!("http://{}", addr)).await.unwrap();

    let key = "invalid_key".to_string();

    let request = GetValueRequest { key };

    let response = client.get_value(request).await;

    assert!(response.is_err());

    if let Err(status) = response {
        assert_eq!(Code::NotFound, status.code());
    }
}

#[tokio::test]
async fn insert_value_should_return_success() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let service = BackendService::new();

    tokio::spawn(async move {
        Server::builder()
            .add_service(KvServer::new(service))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .unwrap();
    });

    let mut client = KvClient::connect(format!("http://{}", addr)).await.unwrap();

    let key = "key1".to_string();
    let value = "value1".to_string();

    let request = InsertValueRequest {
        key: key.clone(),
        value,
    };
    let response = client.insert_value(request).await.unwrap();

    assert!(response.into_inner().success);

    let request = GetValueRequest { key };
    let response = client.get_value(request).await.unwrap();

    assert_eq!("value1", response.into_inner().value);
}
