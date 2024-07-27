# # Use the official Rust image as a parent image
# FROM rust:1.80 as builder
# # Set the working directory in the container
# WORKDIR /usr/src/robopoker
# # Copy the entire project
# COPY . .
# # Install SQLx CLI
# RUN cargo install sqlx-cli
# # Build the application
# RUN cargo build --release
# # Start a new stage with a minimal image
# FROM debian:buster-slim
# # Install OpenSSL and ca-certificates (required for SQLx)
# RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*
# # Copy the binary from the builder stage
# COPY --from=builder /usr/src/robopoker/target/release/robopoker /usr/local/bin/robopoker
# # Copy SQLx CLI from the builder stage
# COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
# # Copy migrations directory
# COPY ./migrations /migrations
# # Set the working directory
# WORKDIR /
# # Set the command to run migrations and then start the application
# CMD sqlx migrate run && robopoker


# Use the official Rust image
FROM rust:1.80

# Set the working directory in the container
WORKDIR /usr/src/robopoker

# Copy the entire project
COPY . .

# Install SQLx CLI and any needed packages
RUN apt-get update && \
    apt-get install -y openssl ca-certificates && \
    rm -rf /var/lib/apt/lists/* && \
    cargo install sqlx-cli

# Set the command to build the project, run migrations, and start the application
CMD cargo build --release && \
    sqlx migrate run && \