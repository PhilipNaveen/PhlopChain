# Use official Rust image as base
FROM rust:1.81-slim

# Set working directory
WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Copy source code
COPY . .

# Build the application
RUN cargo build --release --bin web_server

# Expose port
EXPOSE 3030

# Set environment for better logging
ENV RUST_LOG=debug
ENV RUST_BACKTRACE=1

# Run the web server
CMD ["./target/release/web_server"]
