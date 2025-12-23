# Build stage
FROM rust:1.75-bookworm as builder

WORKDIR /app

# Install RocksDB dependencies
RUN apt-get update && apt-get install -y \
    libclang-dev \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY src ./src

# Build release binary
RUN cargo build --release --bin naseejmesh-gateway

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/naseejmesh-gateway /usr/local/bin/

# Create data directory for embedded DB
RUN mkdir -p /app/data

# Set environment defaults
ENV PORT=8080
ENV HOST=0.0.0.0
ENV SURREAL_PATH=/app/data/gateway.db
ENV RUST_LOG=info

EXPOSE 8080

# Run gateway
CMD ["naseejmesh-gateway"]
