FROM rust:latest AS builder

# Build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Copy source code and assets
COPY src ./src
COPY public ./public

# Build release binary (limit parallelism to reduce memory usage)
ENV CARGO_BUILD_JOBS=1
RUN cargo build --release

FROM debian:bookworm-slim

# Runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/poe-gem-calculator /app/poe-gem-calculator
COPY --from=builder /app/public /app/public
RUN mkdir -p /app/cache

ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

EXPOSE 3000

CMD ["/app/poe-gem-calculator", "--port", "3000", "--host", "0.0.0.0", "--log-level", "info"]
