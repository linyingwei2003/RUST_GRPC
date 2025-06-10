use grpc_demo_proto::{greeter_service_client::GreeterServiceClient, HelloRequest};
use clap::Parser;
use std::time::{Duration, Instant};
use tracing::{info, warn, error};
use tonic::transport::Channel;

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    info!("Debugging error patterns: {} clients making {} requests each", 
          args.clients, args.requests);

    let start_time = Instant::now();
    let mut handles = Vec::new();

    for client_id in 0..args.clients {
        let server_url = args.server.clone();
        let requests = args.requests;
        
        let handle = tokio::spawn(async move {
            benchmark_client(client_id, server_url, requests).await
        });
        handles.push(handle);
    }

    let mut total_requests = 0;
    let mut total_errors = 0;
    let mut error_details = Vec::new();

    for handle in handles {
        let result = handle.await?;
        total_requests += result.requests;
        total_errors += result.errors;
        error_details.extend(result.error_details);
    }

    let total_duration = start_time.elapsed();
    let success_rate = (total_requests - total_errors) as f64 / total_requests as f64 * 100.0;

    println!("\n=== Error Analysis Results ===");
    println!("Total requests: {}", total_requests);
    println!("Total errors: {}", total_errors);
    println!("Success rate: {:.4}%", success_rate);
    println!("Total duration: {:?}", total_duration);

    if !error_details.is_empty() {
        println!("\n=== Error Breakdown ===");
        let mut error_counts = std::collections::HashMap::new();
        for error in &error_details {
            *error_counts.entry(error.clone()).or_insert(0) += 1;
        }
        
        for (error, count) in error_counts {
            println!("  {}: {} occurrences", error, count);
        }
    }

    Ok(())
}

#[derive(Debug)]
struct BenchmarkResult {
    requests: usize,
    errors: usize,
    error_details: Vec<String>,
}

async fn benchmark_client(
    client_id: usize,
    server_url: String,
    requests: usize,
) -> BenchmarkResult {
    let mut error_details = Vec::new();
    
    // Create basic connection
    let channel_result = Channel::from_shared(server_url)
        .unwrap()
        .timeout(Duration::from_secs(60))
        .connect()
        .await;

    let mut client = match channel_result {
        Ok(channel) => GreeterServiceClient::new(channel),
        Err(e) => {
            error!("Client {} failed to connect: {:?}", client_id, e);
            error_details.push(format!("Connection failed: {}", e));
            return BenchmarkResult {
                requests,
                errors: requests,
                error_details,
            };
        }
    };

    let mut errors = 0;

    for i in 0..requests {
        let request = tonic::Request::new(HelloRequest {
            name: format!("Client-{}-Request-{}", client_id, i),
        });
        
        match client.say_hello(request).await {
            Ok(_) => {},
            Err(e) => {
                errors += 1;
                let error_msg = format!("Client {} req {}: {}", client_id, i, e);
                error_details.push(error_msg.clone());
                if errors <= 5 {
                    warn!("{}", error_msg);
                }
            }
        }
    }

    info!("Client {} completed {} requests with {} errors", 
          client_id, requests, errors);

    BenchmarkResult {
        requests,
        errors,
        error_details,
    }
}
