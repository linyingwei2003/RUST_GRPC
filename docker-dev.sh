#!/bin/bash
# Docker development helper script

set -e

echo "🐳 Rust gRPC Docker Development Setup"
echo "====================================="

# Function to check if Docker is running
check_docker() {
    if ! docker info >/dev/null 2>&1; then
        echo "❌ Docker is not running. Please start Docker and try again."
        exit 1
    fi
    echo "✅ Docker is running"
}

# Function to build images
build_images() {
    echo "🔨 Building Docker images..."
    docker-compose build --no-cache
    echo "✅ Images built successfully"
}

# Function to start development environment
start_dev() {
    echo "🚀 Starting development environment..."
    docker-compose up -d rust-grpc-dev
    echo "✅ Development environment started"
    echo "📊 Connect with: docker exec -it rust-grpc-dev bash"
}

# Function to run pprof server
run_pprof_server() {
    echo "🔥 Starting pprof-enabled gRPC server..."
    docker exec -it rust-grpc-dev bash -c "
        echo '🔨 Building pprof server...' &&
        cargo build --release --bin grpc-demo-server-pprof &&
        echo '🚀 Starting server with pprof...' &&
        cargo run --release --bin grpc-demo-server-pprof
    "
}

# Function to run load test
run_load_test() {
    echo "💪 Starting load test..."
    docker exec -it rust-grpc-dev bash -c "
        echo '🔨 Building benchmark tool...' &&
        cargo build --release --bin grpc-demo-benchmark &&
        echo '📊 Running load test...' &&
        cargo run --release --bin grpc-demo-benchmark -- --clients 20 --requests 1000
    "
}

# Function to open shell
open_shell() {
    echo "🐚 Opening shell in development container..."
    docker exec -it rust-grpc-dev bash
}

# Function to stop environment
stop_dev() {
    echo "🛑 Stopping development environment..."
    docker-compose down
    echo "✅ Environment stopped"
}

# Function to clean up
cleanup() {
    echo "🧹 Cleaning up Docker resources..."
    docker-compose down -v
    docker system prune -f
    echo "✅ Cleanup completed"
}

# Function to show logs
show_logs() {
    echo "📝 Showing container logs..."
    docker-compose logs -f rust-grpc-dev
}

# Main menu
show_menu() {
    echo ""
    echo "Available commands:"
    echo "1. build     - Build Docker images"
    echo "2. start     - Start development environment"
    echo "3. pprof     - Run pprof-enabled server"
    echo "4. load      - Run load test"
    echo "5. shell     - Open shell in container"
    echo "6. logs      - Show container logs"
    echo "7. stop      - Stop environment"
    echo "8. clean     - Clean up all resources"
    echo "9. help      - Show this menu"
    echo ""
}

# Check command line arguments
case ${1:-""} in
    "build")
        check_docker
        build_images
        ;;
    "start")
        check_docker
        start_dev
        ;;
    "pprof")
        check_docker
        run_pprof_server
        ;;
    "load")
        check_docker
        run_load_test
        ;;
    "shell")
        check_docker
        open_shell
        ;;
    "logs")
        check_docker
        show_logs
        ;;
    "stop")
        check_docker
        stop_dev
        ;;
    "clean")
        check_docker
        cleanup
        ;;
    "help"|"")
        show_menu
        ;;
    *)
        echo "❌ Unknown command: $1"
        show_menu
        exit 1
        ;;
esac
