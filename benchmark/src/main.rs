use grpc_demo_proto::{greeter_service_client::GreeterServiceClient, HelloRequest};
use clap::Parser;
use std::time::{Duration, Instant};
use tracing::{info, warn};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of concurrent clients
    #[arg(short, long, default_value_t = 10)]
    clients: usize,

    /// Total number of requests per client
    #[arg(short, long, default_value_t = 100)]
    requests: usize,

    /// Server address
    #[arg(short, long, default_value = "http://[::1]:50051")]
    server: String,

    /// Test streaming requests instead of unary
    #[arg(short = 'S', long)]
    streaming: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    info!("Starting benchmark with {} clients making {} requests each", 
          args.clients, args.requests);

    let start_time = Instant::now();
    let mut handles = Vec::new();

    for client_id in 0..args.clients {
        let server_url = args.server.clone();
        let requests = args.requests;
        let streaming = args.streaming;
        
        let handle = tokio::spawn(async move {
            benchmark_client(client_id, server_url, requests, streaming).await
        });
        handles.push(handle);
    }

    let mut total_requests = 0;
    let mut total_errors = 0;
    let mut min_latency = Duration::MAX;
    let mut max_latency = Duration::ZERO;
    let mut total_latency = Duration::ZERO;

    for handle in handles {
        let result = handle.await?;
        total_requests += result.requests;
        total_errors += result.errors;
        min_latency = min_latency.min(result.min_latency);
        max_latency = max_latency.max(result.max_latency);
        total_latency += result.total_latency;
    }

    let total_duration = start_time.elapsed();
    let avg_latency = total_latency / total_requests as u32;
    let qps = total_requests as f64 / total_duration.as_secs_f64();

    println!("\n=== Benchmark Results ===");
    println!("Total requests: {}", total_requests);
    println!("Total errors: {}", total_errors);
    println!("Success rate: {:.2}%", 
             (total_requests - total_errors) as f64 / total_requests as f64 * 100.0);
    println!("Total duration: {:?}", total_duration);
    println!("QPS: {:.2}", qps);
    println!("Latencies:");
    println!("  Min: {:?}", min_latency);
    println!("  Max: {:?}", max_latency);
    println!("  Avg: {:?}", avg_latency);

    Ok(())
}

#[derive(Debug)]
struct BenchmarkResult {
    requests: usize,
    errors: usize,
    min_latency: Duration,
    max_latency: Duration,
    total_latency: Duration,
}

async fn benchmark_client(
    client_id: usize,
    server_url: String,
    requests: usize,
    streaming: bool,
) -> BenchmarkResult {
    let mut client = match GreeterServiceClient::connect(server_url).await {
        Ok(client) => client,
        Err(e) => {
            warn!("Client {} failed to connect: {}", client_id, e);
            return BenchmarkResult {
                requests: 0,
                errors: requests,
                min_latency: Duration::ZERO,
                max_latency: Duration::ZERO,
                total_latency: Duration::ZERO,
            };
        }
    };

    let mut errors = 0;
    let mut min_latency = Duration::MAX;
    let mut max_latency = Duration::ZERO;
    let mut total_latency = Duration::ZERO;

    for i in 0..requests {
        let start = Instant::now();
        
        let result = if streaming {
            // For streaming, we'll just initiate the stream and count the first response
            let request = tonic::Request::new(HelloRequest {
                name: format!("Client-{}-Request-{}", client_id, i),
            });
            
            match client.say_hello_stream(request).await {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        } else {
            let request = tonic::Request::new(HelloRequest {
                name: format!("Client-{}-Request-{}", client_id, i),
            });
            
            match client.say_hello(request).await {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        };

        let latency = start.elapsed();
        total_latency += latency;
        min_latency = min_latency.min(latency);
        max_latency = max_latency.max(latency);

        if result.is_err() {
            errors += 1;
            warn!("Client {} request {} failed: {:?}", client_id, i, result.err());
        }

        // Small delay between requests to avoid overwhelming
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    info!("Client {} completed {} requests with {} errors", 
          client_id, requests, errors);

    BenchmarkResult {
        requests,
        errors,
        min_latency,
        max_latency,
        total_latency,
    }
}
