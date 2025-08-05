FROM rust:latest

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Copy public files
COPY public ./public

# Create cache directory
RUN mkdir -p cache

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Expose the port the app runs on
EXPOSE 3000

# Build and run the application in one step
CMD ["sh", "-c", "cargo build --release && ./target/release/poe-gem-calculator --port 3000 --host 0.0.0.0 --log-level info"]
