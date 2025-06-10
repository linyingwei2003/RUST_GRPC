# Rust gRPC Demo

A simple demonstration of gRPC in Rust using Tonic, featuring both unary and streaming RPC calls.

## Project Structure

```
rust_grpc/
├── proto/          # Protocol buffer definitions and generated code
├── server/         # gRPC server implementation
├── client/         # gRPC client implementation
└── Cargo.toml      # Workspace configuration
```

## Features

- **Unary RPC**: Simple request-response pattern
- **Server Streaming RPC**: Server sends multiple responses for a single request
- **Protocol Buffers**: Uses protobuf for message serialization
- **Async/Await**: Built with Tokio for async runtime

## Proto Definition

The service defines two RPC methods:
- `SayHello`: Returns a single greeting message
- `SayHelloStream`: Returns a stream of greeting messages

## Building and Running

### Prerequisites

Make sure you have Rust installed. If not, install it from [rustup.rs](https://rustup.rs/).

### Build the project

```bash
cargo build
```

### Run the server

In one terminal:
```bash
cargo run --bin grpc-demo-server
```

The server will start listening on `[::1]:50051`.

### Run the client

In another terminal:
```bash
cargo run --bin grpc-demo-client
```

The client will connect to the server, make a unary call, and then a streaming call.

## Expected Output

### Server Output
```
GreeterServer listening on [::1]:50051
Got a request: Request { metadata: MetadataMap { headers: {"te": "trailers", "content-type": "application/grpc", "user-agent": "tonic/0.11.0"} }, message: HelloRequest { name: "World" }, extensions: Extensions }
Got a streaming request: Request { metadata: MetadataMap { headers: {"te": "trailers", "content-type": "application/grpc", "user-agent": "tonic/0.11.0"} }, message: HelloRequest { name: "Streaming World" }, extensions: Extensions }
```

### Client Output
```
=== Single Request ===
RESPONSE=HelloResponse { message: "Hello World!" }

=== Streaming Request ===
STREAM RESPONSE=HelloResponse { message: "Hello Streaming World (message #1)!" }
STREAM RESPONSE=HelloResponse { message: "Hello Streaming World (message #2)!" }
STREAM RESPONSE=HelloResponse { message: "Hello Streaming World (message #3)!" }
STREAM RESPONSE=HelloResponse { message: "Hello Streaming World (message #4)!" }
STREAM RESPONSE=HelloResponse { message: "Hello Streaming World (message #5)!" }
```

## Dependencies

- **tonic**: gRPC implementation for Rust
- **prost**: Protocol Buffers implementation
- **tokio**: Async runtime
- **tokio-stream**: Stream utilities for Tokio

## Learning Resources

- [Tonic Documentation](https://docs.rs/tonic/)
- [gRPC Official Documentation](https://grpc.io/docs/)
- [Protocol Buffers Documentation](https://developers.google.com/protocol-buffers)
