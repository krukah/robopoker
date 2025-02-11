# Build stage
FROM rust:1.80 AS builder
WORKDIR /usr/src/robopoker
COPY . .
RUN cargo build --release

# Final stage
FROM debian:bookworm-slim
WORKDIR /app

# Install PostgreSQL and dependencies
RUN apt-get update && \
    apt-get install -y postgresql libpq-dev libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the Rust binary
COPY --from=builder /usr/src/robopoker/target/release/robopoker .
COPY pgcopy.* .

# Initialize PostgreSQL data directory
RUN mkdir -p /var/lib/postgresql/data && chown -R postgres:postgres /var/lib/postgresql
USER postgres

# Set PostgreSQL data directory and initialize it
ENV PGDATA=/var/lib/postgresql/data
RUN initdb --encoding=UTF8 --locale=C

# Configure PostgreSQL to use Unix sockets
RUN echo "listen_addresses = ''" >> /var/lib/postgresql/data/postgresql.conf && \
    echo "unix_socket_directories = '/var/run/postgresql'" >> /var/lib/postgresql/data/postgresql.conf

# Switch back to root for startup script execution
USER root

# Set up the database and expose the connection URL as an environment variable
ENV DB_URL=postgres:///robopoker?host=/var/run/postgresql

CMD service postgresql start && \
    sleep 5 && \
    PGPASSWORD=postgres psql -U postgres -d postgres -c "CREATE DATABASE robopoker;" && \
    PGPASSWORD=postgres psql -U postgres -d robopoker -c "ALTER ROLE postgres WITH PASSWORD 'postgres';" && \
    /app/robopoker
