@echo off
REM Docker development helper script for Windows

echo 🐳 Rust gRPC Docker Development Setup
echo =====================================

if "%1"=="" goto menu
if "%1"=="build" goto build
if "%1"=="start" goto start
if "%1"=="pprof" goto pprof
if "%1"=="load" goto load
if "%1"=="shell" goto shell
if "%1"=="logs" goto logs
if "%1"=="stop" goto stop
if "%1"=="clean" goto clean
if "%1"=="help" goto menu
goto unknown

:menu
echo.
echo Available commands:
echo 1. docker-dev.bat build     - Build Docker images
echo 2. docker-dev.bat start     - Start development environment
echo 3. docker-dev.bat pprof     - Run pprof-enabled server
echo 4. docker-dev.bat load      - Run load test
echo 5. docker-dev.bat shell     - Open shell in container
echo 6. docker-dev.bat logs      - Show container logs
echo 7. docker-dev.bat stop      - Stop environment
echo 8. docker-dev.bat clean     - Clean up all resources
echo.
echo Quick start:
echo   docker-dev.bat build
echo   docker-dev.bat start
echo   docker-dev.bat pprof
echo.
goto end

:build
echo 🔨 Building Docker images...
docker-compose build --no-cache
if %errorlevel% neq 0 (
    echo ❌ Build failed!
    goto end
)
echo ✅ Images built successfully
goto end

:start
echo 🚀 Starting development environment...
docker-compose up -d rust-grpc-dev
if %errorlevel% neq 0 (
    echo ❌ Failed to start environment!
    goto end
)
echo ✅ Development environment started
echo 📊 Connect with: docker exec -it rust-grpc-dev bash
echo 🌐 Or run: docker-dev.bat shell
goto end

:pprof
echo 🔥 Starting pprof-enabled gRPC server...
docker exec -it rust-grpc-dev bash -c "echo '🔨 Building pprof server...' && cargo build --release --bin grpc-demo-server-pprof && echo '🚀 Starting server with pprof...' && cargo run --release --bin grpc-demo-server-pprof"
goto end

:load
echo 💪 Starting load test...
docker exec -it rust-grpc-dev bash -c "echo '🔨 Building benchmark tool...' && cargo build --release --bin grpc-demo-benchmark && echo '📊 Running load test...' && cargo run --release --bin grpc-demo-benchmark -- --clients 20 --requests 1000"
goto end

:shell
echo 🐚 Opening shell in development container...
docker exec -it rust-grpc-dev bash
goto end

:logs
echo 📝 Showing container logs...
docker-compose logs -f rust-grpc-dev
goto end

:stop
echo 🛑 Stopping development environment...
docker-compose down
if %errorlevel% neq 0 (
    echo ❌ Failed to stop environment!
    goto end
)
echo ✅ Environment stopped
goto end

:clean
echo 🧹 Cleaning up Docker resources...
docker-compose down -v
docker system prune -f
echo ✅ Cleanup completed
goto end

:unknown
echo ❌ Unknown command: %1
goto menu

:end
echo.
