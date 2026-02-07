# Build stage
FROM rust:1.92-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy source to cache dependencies
RUN mkdir -p crates && \
    echo "fn main() {}" > crates/main.rs && \
    echo "pub fn add(a: i64, b: i64) -> i64 { a + b }" > crates/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && \
    rm -rf crates/

# Copy actual source code
COPY crates/ ./crates/

# Build actual binary
RUN cargo build --release

# Runtime stage - use distroless for minimal attack surface
FROM gcr.io/distroless/cc-debian12:latest

# Copy binary from builder
COPY --from=builder /app/target/release/rust_template /usr/local/bin/rust_template

# Set non-root user
USER nonroot:nonroot

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/rust_template", "--version"]

# Run the binary
ENTRYPOINT ["/usr/local/bin/rust_template"]
