use grpc_demo_proto::{
    greeter_service_server::{GreeterService, GreeterServiceServer},
    HelloRequest, HelloResponse,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{info, instrument};
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use axum::{
    extract::Query,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::collections::HashMap;
use tower::ServiceBuilder;
use tower_http::timeout::TimeoutLayer;

#[cfg(target_family = "unix")]
use pprof::{ProfilerGuard, protos::Message};

#[derive(Debug, Default)]
pub struct PprofGreeter {
    request_count: Arc<AtomicU64>,
    stream_count: Arc<AtomicU64>,
}

#[tonic::async_trait]
impl GreeterService for PprofGreeter {
    #[instrument(skip(self))]
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        let start_time = Instant::now();
        let count = self.request_count.fetch_add(1, Ordering::Relaxed) + 1;
        
        // CPU-intensive work for profiling
        let mut sum = 0u64;
        for i in 0..50000 {
            sum = sum.wrapping_add(i * i);
        }
        
        let reply = HelloResponse {
            message: format!("Hello {} (sum: {})!", request.into_inner().name, sum % 1000),
        };

        let duration = start_time.elapsed();
        if count % 100 == 0 {
            info!("Request #{} completed in {:?}", count, duration);
        }

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
        
        let name = request.into_inner().name;
        let (tx, rx) = tokio::sync::mpsc::channel(4);

        tokio::spawn(async move {
            for i in 0..5 {
                // CPU work for each stream message
                let mut sum = 0u64;
                for j in 0..25000 {
                    sum = sum.wrapping_add(j * j * (i + 1));
                }
                
                let response = HelloResponse {
                    message: format!("Hello {} (message #{}, sum: {})!", name, i + 1, sum % 1000),
                };
                
                if tx.send(Ok(response)).await.is_err() {
                    break;
                }
                
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            info!("Streaming request #{} completed in {:?}", count, start_time.elapsed());
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

// pprof HTTP handlers
#[cfg(target_family = "unix")]
async fn pprof_profile(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let seconds = params
        .get("seconds")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(30);

    info!("Starting CPU profiling for {} seconds", seconds);
    
    match pprof::ProfilerGuard::new(100) {
        Ok(guard) => {
            // Wait for the specified duration
            tokio::time::sleep(tokio::time::Duration::from_secs(seconds)).await;
            
            match guard.report().build() {
                Ok(report) => {
                    match report.pprof() {
                        Ok(profile) => {
                            let mut body = Vec::new();
                            if profile.encode(&mut body).is_ok() {
                                info!("CPU profile completed, {} bytes", body.len());
                                (
                                    StatusCode::OK,
                                    [("content-type", "application/octet-stream"),
                                     ("content-disposition", "attachment; filename=\"profile.pb\"")],
                                    body,
                                )
                            } else {
                                (
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    [("content-type", "text/plain"),
                                     ("content-disposition", "")],
                                    b"Failed to encode profile".to_vec(),
                                )
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to generate pprof: {}", e);
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                [("content-type", "text/plain"),
                                 ("content-disposition", "")],
                                format!("Failed to generate pprof: {}", e).into_bytes(),
                            )
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to build profile: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        [("content-type", "text/plain"),
                         ("content-disposition", "")],
                        format!("Error: {}", e).into_bytes(),
                    )
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to create profiler: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("content-type", "text/plain"),
                 ("content-disposition", "")],
                format!("Failed to create profiler: {}", e).into_bytes(),
            )
        }
    }
}

#[cfg(not(target_family = "unix"))]
async fn pprof_profile(Query(_params): Query<HashMap<String, String>>) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        [("content-type", "text/plain")],
        b"pprof profiling is only available on Unix/Linux systems. Use Docker for development.".to_vec(),
    )
}

#[cfg(target_family = "unix")]
async fn pprof_flamegraph(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let seconds = params
        .get("seconds")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(30);

    info!("Generating flamegraph for {} seconds", seconds);
    
    match pprof::ProfilerGuard::new(100) {
        Ok(guard) => {
            // Wait for the specified duration
            tokio::time::sleep(tokio::time::Duration::from_secs(seconds)).await;
              match guard.report().build() {
                Ok(report) => {
                    let mut flamegraph_data = Vec::new();
                    match report.flamegraph(&mut flamegraph_data) {
                        Ok(_) => {
                            info!("Flamegraph generated, {} bytes", flamegraph_data.len());
                            (
                                StatusCode::OK,
                                [("content-type", "image/svg+xml"),
                                 ("content-disposition", "attachment; filename=\"flamegraph.svg\"")],
                                flamegraph_data,
                            )
                        }
                        Err(e) => {
                            tracing::error!("Failed to generate flamegraph: {}", e);
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                [("content-type", "text/plain"),
                                 ("content-disposition", "")],
                                format!("Error generating flamegraph: {}", e).into_bytes(),
                            )
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to build profile: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        [("content-type", "text/plain"),
                         ("content-disposition", "")],
                        format!("Error: {}", e).into_bytes(),
                    )
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to create profiler: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("content-type", "text/plain"),
                 ("content-disposition", "")],
                format!("Failed to create profiler: {}", e).into_bytes(),
            )
        }
    }
}

#[cfg(not(target_family = "unix"))]
async fn pprof_flamegraph(Query(_params): Query<HashMap<String, String>>) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        [("content-type", "text/plain")],
        b"Flamegraph generation is only available on Unix/Linux systems. Use Docker for development.".to_vec(),
    )
}

async fn pprof_heap() -> impl IntoResponse {
    info!("Heap profile requested");
    
    #[cfg(target_family = "unix")]
    {
        let message = "Heap profiling in Rust requires external tools like valgrind or heaptrack.\nFor memory analysis, use: valgrind --tool=massif ./target/release/grpc-demo-server-pprof";
        (
            StatusCode::OK,
            [("content-type", "text/plain")],
            message.as_bytes().to_vec(),
        )
    }
    
    #[cfg(not(target_family = "unix"))]
    {
        let message = "Heap profiling is only available on Unix/Linux systems. Use Docker for development.";
        (
            StatusCode::NOT_IMPLEMENTED,
            [("content-type", "text/plain")],
            message.as_bytes().to_vec(),
        )
    }
}

async fn pprof_index() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>🔥 Rust gRPC pprof Profiling</title>
    <style>
        body { font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 20px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); min-height: 100vh; }
        .container { max-width: 900px; margin: 0 auto; background: rgba(255,255,255,0.95); padding: 30px; border-radius: 15px; box-shadow: 0 10px 30px rgba(0,0,0,0.2); }
        h1 { color: #333; text-align: center; margin-bottom: 30px; font-size: 2.5em; }
        .subtitle { text-align: center; color: #666; margin-bottom: 30px; font-size: 1.2em; }
        .endpoint { margin: 20px 0; padding: 20px; border: none; border-radius: 10px; background: linear-gradient(45deg, #f0f2f5, #ffffff); box-shadow: 0 4px 6px rgba(0,0,0,0.1); }
        .endpoint h3 { margin-top: 0; color: #444; font-size: 1.4em; }
        a { color: #667eea; text-decoration: none; font-weight: bold; padding: 8px 16px; background: rgba(102, 126, 234, 0.1); border-radius: 5px; display: inline-block; margin: 5px; transition: all 0.3s ease; }
        a:hover { background: #667eea; color: white; transform: translateY(-2px); box-shadow: 0 4px 8px rgba(102, 126, 234, 0.3); }
        .description { color: #666; margin: 15px 0; line-height: 1.6; }
        .stats { background: linear-gradient(45deg, #667eea, #764ba2); color: white; padding: 20px; border-radius: 10px; margin: 20px 0; }
        .stats strong { color: #fff; }
        pre { background: #f8f9fa; padding: 15px; border-radius: 8px; overflow-x: auto; border-left: 4px solid #667eea; }
        .warning { background: #fff3cd; border: 1px solid #ffeaa7; padding: 15px; border-radius: 8px; margin: 20px 0; color: #856404; }
        .success { background: #d1edff; border: 1px solid #74b9ff; padding: 15px; border-radius: 8px; margin: 20px 0; color: #0984e3; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🔥 Rust gRPC pprof Profiling</h1>
        <p class="subtitle">Professional CPU profiling for Rust gRPC services</p>
        
        <div class="stats">
            <strong>🚀 Server Status:</strong> Running with pprof enabled<br>
            <strong>🌐 gRPC Endpoint:</strong> [::1]:50051<br>
            <strong>📊 Profiling Server:</strong> [::1]:3000<br>
            <strong>🐳 Environment:</strong> <span id="platform">Detecting...</span>
        </div>

        <div class="success">
            <strong>✅ Quick Start:</strong><br>
            1. Start load testing: <code>cargo run --release --bin grpc-demo-benchmark</code><br>
            2. Click profiling links below while load is running<br>
            3. Analyze downloaded profiles
        </div>

        <div class="endpoint">
            <h3>🔥 CPU Profile (pprof format)</h3>
            <div class="description">Generate CPU profile in pprof format for analysis with <code>go tool pprof</code></div>
            <a href="/debug/pprof/profile?seconds=30">📊 30 seconds</a>
            <a href="/debug/pprof/profile?seconds=60">📊 60 seconds</a>
            <a href="/debug/pprof/profile?seconds=120">📊 120 seconds</a>
        </div>

        <div class="endpoint">
            <h3>🔥 Flamegraph (Interactive SVG)</h3>
            <div class="description">Generate interactive flamegraph visualization for immediate analysis</div>
            <a href="/debug/pprof/flamegraph?seconds=30">🔥 30 seconds</a>
            <a href="/debug/pprof/flamegraph?seconds=60">🔥 60 seconds</a>
            <a href="/debug/pprof/flamegraph?seconds=120">🔥 120 seconds</a>
        </div>

        <div class="endpoint">
            <h3>🧠 Memory Analysis</h3>
            <div class="description">Memory profiling information and tools</div>
            <a href="/debug/pprof/heap">🧠 Heap Info</a>
        </div>

        <h2>🛠️ Analysis Tools</h2>
        
        <h3>Command Line Usage</h3>
        <pre># Download CPU profile
curl -o profile.pb "http://localhost:3000/debug/pprof/profile?seconds=60"

# Download flamegraph
curl -o flamegraph.svg "http://localhost:3000/debug/pprof/flamegraph?seconds=60"

# Analyze with go tool pprof (if installed)
go tool pprof profile.pb

# Interactive web interface
go tool pprof -http=:8081 profile.pb</pre>

        <h3>🐳 Docker Usage</h3>
        <pre># Start development environment
docker-compose up -d rust-grpc-dev

# Connect to container
docker exec -it rust-grpc-dev bash

# Build and run pprof server
cargo build --release --bin grpc-demo-server-pprof
cargo run --release --bin grpc-demo-server-pprof

# Generate load from another terminal
docker exec -it rust-grpc-dev bash
cargo run --release --bin grpc-demo-benchmark -- --clients 20 --requests 1000</pre>

        <div class="warning">
            <strong>⚠️ Note:</strong> pprof profiling requires Linux. On Windows, use the Docker development environment for full functionality.
        </div>

        <script>
            // Detect platform
            document.getElementById('platform').textContent = 
                navigator.platform.includes('Win') ? 'Windows (Limited)' : 
                navigator.platform.includes('Mac') ? 'macOS' : 'Linux/Unix';
        </script>
    </div>
</body>
</html>
    "#)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing with reduced verbosity for performance
    tracing_subscriber::fmt()
        .with_env_filter("grpc_demo_server=info")
        .with_target(false)
        .init();

    let greeter = PprofGreeter::default();
    
    // Setup pprof HTTP server
    let pprof_app = Router::new()
        .route("/", get(pprof_index))
        .route("/debug/pprof/profile", get(pprof_profile))
        .route("/debug/pprof/heap", get(pprof_heap))
        .route("/debug/pprof/flamegraph", get(pprof_flamegraph));

    // Start HTTP server for pprof
    let http_addr = "[::]:3000";
    info!("🔥 pprof HTTP server starting on http://{}", http_addr);
    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(http_addr).await.unwrap();
        axum::serve(listener, pprof_app).await.unwrap();
    });    // Start gRPC server with connection pooling
    let grpc_addr = "[::]:50051".parse()?;
    info!("🚀 gRPC server with pprof profiling and connection pooling listening on {}", grpc_addr);
    info!("📊 Profiling dashboard: http://localhost:3000");
    
    #[cfg(target_family = "unix")]
    info!("🔥 pprof ready: CPU profiling and flamegraphs available");
    
    #[cfg(not(target_family = "unix"))]
    info!("⚠️  pprof limited: Use Docker for full profiling capabilities");    // Configure server for 100% reliability with optimized connection pooling
    Server::builder()
        // Connection pooling configuration
        .tcp_keepalive(Some(Duration::from_secs(600))) // 10 minutes keepalive
        .tcp_nodelay(true) // Disable Nagle's algorithm for lower latency
        .timeout(Duration::from_secs(120)) // Longer timeout for reliability
        .concurrency_limit_per_connection(10000) // Much higher limit to prevent rejections
        .initial_stream_window_size(Some(16 * 1024 * 1024)) // 16MB initial window for better flow control
        .initial_connection_window_size(Some(16 * 1024 * 1024)) // 16MB connection window
        .max_concurrent_streams(Some(10000)) // Much higher stream limit
        .http2_keepalive_interval(Some(Duration::from_secs(60))) // HTTP/2 keepalive
        .http2_keepalive_timeout(Some(Duration::from_secs(20))) // HTTP/2 keepalive timeout
        .http2_adaptive_window(Some(true)) // Enable adaptive flow control
        // Add service with middleware
        .layer(
            ServiceBuilder::new()
                .layer(TimeoutLayer::new(Duration::from_secs(120))) // Match server timeout
                .into_inner(),
        )
        .add_service(GreeterServiceServer::new(greeter))
        .serve(grpc_addr)
        .await?;

    Ok(())
}
