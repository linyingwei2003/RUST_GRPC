use grpc_demo_proto::{
    greeter_service_server::{GreeterService, GreeterServiceServer},
    HelloRequest, HelloResponse,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tonic::{transport::Server, Request, Response, Status};
use tracing::{info, instrument};
use tower::ServiceBuilder;
use tower_http::timeout::TimeoutLayer;

pub mod campaign;

#[derive(Debug, Default)]
pub struct MyGreeter {
    request_count: AtomicU64,
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
        
        let name = request.into_inner().name;

        // CPU work for profiling
        let mut sum = 0u64;
        for i in 0..50000 {
            sum = sum.wrapping_add(i * i);
        }

        // Example usage of Campaign bid calculation
        let campaign = campaign::Campaign::new(50.0, 2.5);
        let next_bid = campaign.next_bid();

        let reply = HelloResponse {
            message: format!(
                "Hello {} (optimized server, request #{}, sum: {}, next_bid: {:.2})!", 
                name, count, sum % 1000, next_bid
            ),
        };

        let duration = start_time.elapsed();
        if count % 100 == 0 {
            info!("Request #{} completed in {:?}, next_bid: {:.2}", count, duration, next_bid);
        }

        Ok(Response::new(reply))
    }

    type SayHelloStreamStream = tonic::codec::Streaming<HelloResponse>;

    async fn say_hello_stream(
        &self,
        _request: Request<HelloRequest>,
    ) -> Result<Response<Self::SayHelloStreamStream>, Status> {
        Err(Status::unimplemented("Streaming not implemented in main server"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("grpc_demo_server=info,tower=info,tonic=info")
        .with_target(false)
        .init();

    let addr = "[::1]:50051".parse()?;
    let greeter = MyGreeter::default();

    info!("Enhanced GreeterServer with Connection Pooling listening on {}", addr);
    info!("Server features: request counting, detailed logging, connection pooling");

    Server::builder()
        .tcp_keepalive(Some(Duration::from_secs(600)))
        .tcp_nodelay(true)
        .timeout(Duration::from_secs(30))
        .concurrency_limit_per_connection(256)
        .initial_stream_window_size(Some(1024 * 1024))
        .initial_connection_window_size(Some(1024 * 1024))
        .max_concurrent_streams(Some(1000))
        .layer(
            ServiceBuilder::new()
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                .into_inner(),
        )
        .add_service(GreeterServiceServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}