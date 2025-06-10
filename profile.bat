@echo off
REM Rust gRPC Server Profiling Scripts

echo Setting up environment...
set PROTOC=%~dp0protoc\bin\protoc.exe
set RUST_LOG=info

echo.
echo ================================
echo  Rust gRPC Profiling Suite
echo ================================
echo.
echo 1. Build all components
echo 2. Start enhanced server (with metrics)
echo 3. Run basic benchmark
echo 4. Run heavy load test
echo 5. Start CPU profiling server
echo 6. View metrics (opens browser)
echo 7. Generate flame graph report
echo 8. Clean build artifacts
echo 9. Exit
echo.

:menu
set /p choice="Choose an option (1-9): "

if "%choice%"=="1" goto build
if "%choice%"=="2" goto server
if "%choice%"=="3" goto benchmark_basic
if "%choice%"=="4" goto benchmark_heavy
if "%choice%"=="5" goto profiling
if "%choice%"=="6" goto metrics
if "%choice%"=="7" goto flamegraph
if "%choice%"=="8" goto clean
if "%choice%"=="9" goto exit

echo Invalid choice. Please select 1-9.
goto menu

:build
echo Building all components...
cargo build --release
if %errorlevel% neq 0 (
    echo Build failed!
    pause
    goto menu
)
echo Build completed successfully!
pause
goto menu

:server
echo Starting enhanced gRPC server with metrics...
echo Server will be available at: http://[::1]:50051
echo Metrics will be available at: http://localhost:9090/metrics
echo Press Ctrl+C to stop the server.
echo.
cargo run --release --bin grpc-demo-server
goto menu

:benchmark_basic
echo Running basic benchmark (10 clients, 100 requests each)...
echo.
cargo run --release --bin grpc-demo-benchmark
echo.
echo Benchmark completed. Check results above.
pause
goto menu

:benchmark_heavy
echo Running heavy load test (50 clients, 500 requests each)...
echo.
cargo run --release --bin grpc-demo-benchmark -- --clients 50 --requests 500
echo.
echo Heavy load test completed. Check results above.
pause
goto menu

:profiling
echo Starting CPU profiling server...
echo Send some load using the benchmark, then press Ctrl+C to generate flamegraph.
echo.
cargo run --release --bin grpc-demo-profiling
echo.
echo Profiling completed. Check for flamegraph.svg and profile.pb files.
pause
goto menu

:metrics
echo Opening metrics dashboard in browser...
start http://localhost:9090/metrics
echo If server is not running, start it first with option 2.
pause
goto menu

:flamegraph
echo Checking for generated profiling files...
if exist flamegraph.svg (
    echo flamegraph.svg found - opening in default browser...
    start flamegraph.svg
) else (
    echo flamegraph.svg not found. Run CPU profiling first (option 5).
)

if exist profile.pb (
    echo profile.pb found (raw profile data for external tools)
) else (
    echo profile.pb not found.
)
pause
goto menu

:clean
echo Cleaning build artifacts...
cargo clean
echo Clean completed.
pause
goto menu

:exit
echo Goodbye!
exit /b 0
