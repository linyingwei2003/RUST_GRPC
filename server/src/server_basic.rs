use grpc_demo_proto::{
    greeter_service_server::{GreeterService, GreeterServiceServer},
    HelloRequest, HelloResponse,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{info, instrument};

#[derive(Debug, Default)]
pub struct BaselineGreeter {
    request_count: AtomicU64,
}

#[tonic::async_trait]
impl GreeterService for BaselineGreeter {
    #[instrument(skip(self))]
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        let start_time = Instant::now();
        let count = self.request_count.fetch_add(1, Ordering::Relaxed) + 1;
        
        let name = request.into_inner().name;

        // Same CPU work as other servers for fair comparison
        let mut sum = 0u64;
        for i in 0..50000 {
            sum = sum.wrapping_add(i * i);
        }

        let reply = HelloResponse {
            message: format!("Hello {} (baseline server, request #{}, sum: {})!", name, count, sum % 1000),
        };

        let duration = start_time.elapsed();
        if count % 100 == 0 {
            info!("Baseline request #{} completed in {:?}", count, duration);
        }

        Ok(Response::new(reply))
    }

    type SayHelloStreamStream = tonic::codec::Streaming<HelloResponse>;

    async fn say_hello_stream(
        &self,
        _request: Request<HelloRequest>,
    ) -> Result<Response<Self::SayHelloStreamStream>, Status> {
        // Simple implementation for baseline
        Err(Status::unimplemented("Streaming not implemented in baseline server"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize basic tracing
    tracing_subscriber::fmt()
        .with_env_filter("grpc_demo_server=info")
        .with_target(false)
        .init();

    let addr = "[::]:50052".parse()?;  // Different port
    let greeter = BaselineGreeter::default();

    info!("🔧 Baseline Server (NO connection pooling) listening on {}", addr);
    info!("📊 Features: Basic gRPC only, no optimizations");

    // Basic server configuration - NO connection pooling optimizations
    Server::builder()
        .add_service(GreeterServiceServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
