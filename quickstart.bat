@echo off
echo 🚀 Rust gRPC Server - Quick Start
echo.

:menu
echo Choose an option:
echo 1. Build all components
echo 2. Start optimized server (port 50051)
echo 3. Start basic server (port 50052)
echo 4. Run benchmark test
echo 5. Run Docker development environment
echo 6. Access profiling dashboard
echo 0. Exit
echo.

set /p choice="Enter your choice (0-6): "

if "%choice%"=="1" goto build
if "%choice%"=="2" goto optimized
if "%choice%"=="3" goto basic
if "%choice%"=="4" goto benchmark
if "%choice%"=="5" goto docker
if "%choice%"=="6" goto profiling
if "%choice%"=="0" goto exit
goto menu

:build
echo Building all components...
cargo build --release
pause
goto menu

:optimized
echo Starting optimized server with connection pooling on port 50051...
cargo run --release --bin grpc-demo-server-optimized
pause
goto menu

:basic
echo Starting basic server on port 50052...
cargo run --release --bin grpc-demo-server-basic
pause
goto menu

:benchmark
echo Running benchmark tests...
echo Testing optimized server:
cargo run --release --bin grpc-demo-benchmark -- --server http://localhost:50051 --clients 20 --requests 25
echo.
echo Testing basic server:
cargo run --release --bin grpc-demo-benchmark -- --server http://localhost:50052 --clients 20 --requests 25
pause
goto menu

:docker
echo Starting Docker development environment...
docker-compose up -d
echo Docker container started. Access with: docker exec -it rust-grpc-dev bash
pause
goto menu

:profiling
echo Opening profiling dashboard...
start http://localhost:3000
echo Profiling dashboard should open in your browser
pause
goto menu

:exit
echo Goodbye! 👋
pause
exit
