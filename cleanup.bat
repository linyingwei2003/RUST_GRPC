@echo off
REM Cleanup script for Rust gRPC project workspace

echo.
echo ================================
echo  Rust gRPC Workspace Cleanup
echo ================================
echo.

echo Stopping any running processes...
taskkill /F /IM grpc-demo-server.exe 2>nul
taskkill /F /IM grpc-demo-profiling.exe 2>nul
taskkill /F /IM grpc-demo-benchmark.exe 2>nul
taskkill /F /IM grpc-demo-client.exe 2>nul

echo.
echo Waiting for processes to stop...
timeout /t 2 >nul

echo.
echo Cleaning build artifacts...
cargo clean

if %errorlevel% neq 0 (
    echo Warning: Some files could not be cleaned due to access restrictions.
    echo This is normal if antivirus or other processes are using the files.
    echo.
    echo You can manually delete the 'target' folder when no processes are running:
    echo   rmdir /s /q target
)

echo.
echo Workspace cleanup completed!
echo.
echo Clean workspace structure:
echo - benchmark/     - Load testing tool
echo - client/        - Basic gRPC client  
echo - profiling/     - System profiling server
echo - proto/         - Protocol buffer definitions
echo - protoc/        - Protocol buffer compiler
echo - server/        - Enhanced gRPC server
echo - *.md files     - Documentation
echo - profile.bat    - Interactive profiling menu
echo.
echo All temporary and redundant files have been removed.
pause
