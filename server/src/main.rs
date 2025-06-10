use grpc_demo_proto::{
    greeter_service_server::{GreeterService, GreeterServiceServer},
    HelloRequest, HelloResponse,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{info, instrument};
use std::time::Instant;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct MyGreeter {
    request_count: Arc<AtomicU64>,
    stream_count: Arc<AtomicU64>,
}

#[tonic::async_trait]
impl GreeterService for MyGreeter {
    #[instrument(skip(self))]
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        let start_time = Instant::now();
        let count = self.request_count.fetch_add(1, Ordering::Relaxed) + 1;
        
        info!("Got request #{}: {:?}", count, request.get_ref().name);

        // Simulate some processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let reply = HelloResponse {
            message: format!("Hello {}!", request.into_inner().name),
        };

        let duration = start_time.elapsed();
        info!("Request #{} completed in {:?}", count, duration);

        Ok(Response::new(reply))
    }

    type SayHelloStreamStream = ReceiverStream<Result<HelloResponse, Status>>;

    #[instrument(skip(self))]
    async fn say_hello_stream(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<Self::SayHelloStreamStream>, Status> {
        let start_time = Instant::now();
        let count = self.stream_count.fetch_add(1, Ordering::Relaxed) + 1;
        
        info!("Got streaming request #{}: {:?}", count, request.get_ref().name);

        let name = request.into_inner().name;
        let (tx, rx) = tokio::sync::mpsc::channel(4);

        tokio::spawn(async move {
            for i in 0..5 {
                let response = HelloResponse {
                    message: format!("Hello {} (message #{})!", name, i + 1),
                };
                
                if tx.send(Ok(response)).await.is_err() {
                    break;
                }
                
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
            info!("Streaming request #{} completed in {:?}", count, start_time.elapsed());
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("grpc_demo_server=info,tower=info,tonic=info")
        .with_target(false)
        .init();

    let addr = "[::1]:50051".parse()?;
    let greeter = MyGreeter::default();

    info!("Enhanced GreeterServer listening on {}", addr);
    info!("Server includes request counting and detailed logging");

    Server::builder()
        .add_service(GreeterServiceServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}