# Build stage
FROM rust:1.80 AS builder
WORKDIR /usr/src/robopoker
COPY . .
RUN cargo build --release

# Binary stage
FROM debian:bookworm-slim AS binary

WORKDIR /output

RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/robopoker/target/release/robopoker /usr/local/bin/robopoker
ENTRYPOINT ["robopoker"]
