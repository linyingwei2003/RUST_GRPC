use grpc_demo_proto::{greeter_service_client::GreeterServiceClient, HelloRequest};
use tokio_stream::StreamExt;
use tonic::transport::{Channel, Endpoint};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create connection with pooling configuration
    let endpoint = Endpoint::from_static("http://[::1]:50051")
        .timeout(Duration::from_secs(30))
        .tcp_keepalive(Some(Duration::from_secs(600)))
        .tcp_nodelay(true)
        .http2_keep_alive_interval(Duration::from_secs(30))
        .keep_alive_while_idle(true);

    let channel = endpoint.connect().await?;
    let mut client = GreeterServiceClient::new(channel);

    println!("=== Connected with Connection Pooling ===");

    // Single request
    println!("=== Single Request ===");
    let request = tonic::Request::new(HelloRequest {
        name: "World".into(),
    });

    let response = client.say_hello(request).await?;
    println!("RESPONSE={:?}", response.into_inner());

    // Multiple requests to test connection reuse
    println!("\n=== Multiple Requests (Connection Reuse Test) ===");
    for i in 1..=5 {
        let request = tonic::Request::new(HelloRequest {
            name: format!("Request {}", i),
        });

        let start = std::time::Instant::now();
        let response = client.say_hello(request).await?;
        let duration = start.elapsed();
        
        println!("Request {}: {:?} (took: {:?})", i, response.into_inner(), duration);
    }

    // Streaming request
    println!("\n=== Streaming Request ===");
    let request = tonic::Request::new(HelloRequest {
        name: "Streaming World".into(),
    });

    let mut stream = client.say_hello_stream(request).await?.into_inner();

    while let Some(response) = stream.next().await {
        match response {
            Ok(hello_response) => {
                println!("STREAM RESPONSE={:?}", hello_response);
            }
            Err(e) => {
                println!("STREAM ERROR={:?}", e);
                break;
            }
        }
    }

    println!("\n=== Connection Pooling Test Complete ===");
    Ok(())
}
