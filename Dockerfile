# Rust gRPC Development with pprof Profiling
FROM rust:1.82-bullseye

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    protobuf-compiler \
    libprotobuf-dev \
    pkg-config \
    libssl-dev \
    curl \
    wget \
    git \
    htop \
    && rm -rf /var/lib/apt/lists/*

# Install Go for pprof analysis tools
RUN wget https://go.dev/dl/go1.23.5.linux-amd64.tar.gz && \
    tar -C /usr/local -xzf go1.23.5.linux-amd64.tar.gz && \
    rm go1.23.5.linux-amd64.tar.gz

# Add Go to PATH
ENV PATH="/usr/local/go/bin:${PATH}"

# Install graphviz for pprof visualization
RUN apt-get update && apt-get install -y graphviz && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /workspace

# Create a non-root user with proper permissions
RUN useradd -m -s /bin/bash rustdev && \
    chown -R rustdev:rustdev /workspace

# Switch to non-root user
USER rustdev

# Copy workspace files
COPY --chown=rustdev:rustdev . .

# Create target directory with proper permissions
RUN mkdir -p target && chmod 755 target

# Pre-build dependencies for faster iteration
RUN cargo fetch

# Expose ports
# 50051 - gRPC server
# 3000 - pprof HTTP server
# 8080 - benchmark dashboard (optional)
EXPOSE 50051 3000 8080

# Default command - interactive shell
CMD ["/bin/bash"]
