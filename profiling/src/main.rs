use grpc_demo_proto::{
    greeter_service_server::{GreeterService, GreeterServiceServer},
    HelloRequest, HelloResponse,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{info, warn};
use sysinfo::{System, Pid};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

#[derive(Debug, Default, Clone)]
pub struct ProfilingGreeter {
    request_count: Arc<AtomicU64>,
    total_cpu_time: Arc<AtomicU64>,
}

#[tonic::async_trait]
impl GreeterService for ProfilingGreeter {    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        let start = Instant::now();
        let count = self.request_count.fetch_add(1, Ordering::Relaxed);
        
        // Simulate some CPU work for profiling
        let mut sum = 0u64;
        for i in 0..10000 {
            sum = sum.wrapping_add(i * i);
        }
        
        let elapsed = start.elapsed();
        self.total_cpu_time.fetch_add(elapsed.as_nanos() as u64, Ordering::Relaxed);
        
        info!("Request #{} processed in {:?} (sum: {})", count, elapsed, sum % 1000);
        
        let reply = HelloResponse {
            message: format!("Hello {} (sum: {})!", request.into_inner().name, sum % 1000),
        };

        Ok(Response::new(reply))
    }

    type SayHelloStreamStream = ReceiverStream<Result<HelloResponse, Status>>;    async fn say_hello_stream(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<Self::SayHelloStreamStream>, Status> {        let name = request.into_inner().name;
        let (tx, rx) = tokio::sync::mpsc::channel(4);

        tokio::spawn(async move {
            for i in 0..5 {
                let start = Instant::now();
                // Simulate CPU work
                let mut sum = 0u64;
                for j in 0..5000 {
                    sum = sum.wrapping_add(j * j * i);
                }
                
                let elapsed = start.elapsed();
                info!("Stream message #{} processed in {:?}", i + 1, elapsed);
                
                let response = HelloResponse {
                    message: format!("Hello {} (message #{}, sum: {})!", name, i + 1, sum % 1000),
                };
                
                if tx.send(Ok(response)).await.is_err() {
                    break;
                }
                
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("Starting Windows-compatible profiling server...");
    info!("This will monitor CPU and memory usage during operation");

    let greeter = Arc::new(ProfilingGreeter::default());
    let greeter_clone = greeter.clone();    // Start system monitoring task
    let monitor_handle = tokio::spawn(async move {
        let mut system = System::new_all();
        let pid = Pid::from_u32(std::process::id());
        
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            system.refresh_all();
            
            if let Some(process) = system.process(pid) {
                let cpu_usage = process.cpu_usage();
                let memory_mb = process.memory() / 1024 / 1024;
                let requests = greeter_clone.request_count.load(Ordering::Relaxed);
                let total_cpu_ns = greeter_clone.total_cpu_time.load(Ordering::Relaxed);
                
                info!("STATS: CPU: {:.1}%, Memory: {}MB, Requests: {}, Avg req time: {}μs",
                      cpu_usage, memory_mb, requests,
                      if requests > 0 { total_cpu_ns / requests / 1000 } else { 0 });
            }
        }
    });

    let addr = "[::1]:50051".parse()?;

    info!("Profiling GreeterServer listening on {}", addr);
    info!("System monitoring active - stats printed every 5 seconds");    // Handle graceful shutdown
    let server = Server::builder()
        .add_service(GreeterServiceServer::new((*greeter).clone()))
        .serve_with_shutdown(addr, async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to listen for ctrl+c");
            info!("Received shutdown signal, stopping monitoring...");
        });

    // Run server and monitor concurrently
    tokio::select! {
        result = server => {
            if let Err(e) = result {
                warn!("Server error: {}", e);
            }
        }
        _ = monitor_handle => {
            info!("Monitor task completed");
        }
    }

    // Print final statistics
    let final_requests = greeter.request_count.load(Ordering::Relaxed);
    let final_cpu_time = greeter.total_cpu_time.load(Ordering::Relaxed);
    
    info!("=== Final Statistics ===");
    info!("Total requests processed: {}", final_requests);
    if final_requests > 0 {
        info!("Average request time: {}μs", final_cpu_time / final_requests / 1000);
    }
    info!("Profiling session complete!");

    Ok(())
}
