# syntax=docker/dockerfile:1
# Build stage — Alpine is musl-native, so `cargo build` targets the musl triple
# and produces a fully STATIC binary (no glibc). That lets the runtime stage be
# distroless/static (no glibc/openssl/libstdc++), eliminating the base-image
# OS-package CVEs that a glibc image (distroless/cc) otherwise carries.
ARG RUST_VERSION=1.92
FROM rust:${RUST_VERSION}-alpine AS builder

# musl-dev + build-base provide the C toolchain/assembler that ring (rustls'
# crypto backend) compiles against; the musl target links it statically.
RUN apk add --no-cache musl-dev build-base

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy source to cache dependencies (build.rs is intentionally NOT
# present yet — the dependency build needs no build script).
RUN mkdir -p crates && \
    echo "fn main() {}" > crates/main.rs && \
    echo "" > crates/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && \
    rm -rf crates/

# Copy actual source code and files referenced by include_str!
# build.rs lives at the repo root and emits NSIP_ERROR_TYPE_URI_BASE via
# cargo:rustc-env, which crates/lib.rs reads with env!(); it MUST be present
# or the real build fails with "environment variable ... not defined".
COPY README.md ./
COPY build.rs ./
COPY crates/ ./crates/

# Invalidate cargo fingerprints so real source is compiled
RUN rm -f target/release/deps/libnsip* \
         target/release/deps/nsip* \
         target/release/nsip && \
    cargo build --release

# Runtime stage — distroless/static (no glibc/openssl) for a static musl binary.
# TLS roots are bundled in the binary (reqwest rustls-tls -> webpki-roots), so
# no system CA certs are required; the base still ships ca-certificates +
# tzdata + a nonroot user for robustness.
FROM gcr.io/distroless/static-debian12:nonroot

# Copy the statically-linked binary from builder
COPY --from=builder /app/target/release/nsip /usr/local/bin/nsip

# Run as the distroless nonroot user (uid 65532)
USER nonroot:nonroot

# Health check
HEALTHCHECK --interval=30s --timeout=3s \
    --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/nsip", "--version"]

# Run the binary
ENTRYPOINT ["/usr/local/bin/nsip"]
