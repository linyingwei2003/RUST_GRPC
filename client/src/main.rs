use grpc_demo_proto::{greeter_service_client::GreeterServiceClient, HelloRequest};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GreeterServiceClient::connect("http://[::1]:50051").await?;

    // Single request
    println!("=== Single Request ===");
    let request = tonic::Request::new(HelloRequest {
        name: "World".into(),
    });

    let response = client.say_hello(request).await?;
    println!("RESPONSE={:?}", response.into_inner());

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

    Ok(())
}
