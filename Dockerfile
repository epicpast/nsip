# Build stage
FROM rust:1.92-slim AS builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy source to cache dependencies
RUN mkdir -p crates && \
    echo "fn main() {}" > crates/main.rs && \
    echo "" > crates/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && \
    rm -rf crates/

# Copy actual source code and files referenced by include_str!
COPY README.md ./
COPY crates/ ./crates/

# Invalidate cargo fingerprints so real source is compiled
RUN rm -f target/release/deps/libnsip* \
         target/release/deps/nsip* \
         target/release/nsip && \
    cargo build --release

# Runtime stage - use distroless for minimal attack surface
FROM gcr.io/distroless/cc-debian12:latest

# Copy binary from builder
COPY --from=builder /app/target/release/nsip /usr/local/bin/nsip

# Set non-root user
USER nonroot:nonroot

# Health check
HEALTHCHECK --interval=30s --timeout=3s \
    --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/nsip", "--version"]

# Run the binary
ENTRYPOINT ["/usr/local/bin/nsip"]
