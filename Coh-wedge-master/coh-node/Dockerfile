# Stage 1: Build
FROM rust:1.75-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy and build Rust project
COPY coh-node/ ./
RUN cargo build --release --package coh-cli

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/coh-validator /usr/local/bin/

# Create non-root user for security
RUN useradd -m -u 1000 verifier
USER verifier

# Default entrypoint
ENTRYPOINT ["coh-validator"]
CMD ["--help"]