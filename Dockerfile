# Build stages for server side

FROM rust:1.90 AS trainer-builder
WORKDIR /app
COPY Cargo.toml         ./
COPY Cargo.lock         ./
COPY src                ./src
COPY benches            ./benches
RUN cargo build --release --bin trainer --features database

FROM rust:1.90 AS analyze-builder
WORKDIR /app
COPY Cargo.toml         ./
COPY Cargo.lock         ./
COPY src                ./src
COPY benches            ./benches
RUN cargo build --release --bin analyze --features database

# Build stages for client side

FROM rust:1.90 AS explore-builder
RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-bindgen-cli
RUN cargo install trunk
WORKDIR /app
COPY Cargo.toml         ./
COPY Cargo.lock         ./
COPY Trunk.toml         ./
COPY index.html         ./
COPY input.css          ./
COPY tailwind.config.js ./
COPY src                ./src
COPY benches            ./benches
COPY .cargo             ./.cargo
RUN trunk build --release

# Runtime stages for server side

FROM debian:bookworm-slim   AS trainer
RUN apt-get update && apt-get install -y coreutils && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=trainer-builder     /app/target/release/trainer     /app/trainer
# TRAIN_DURATION env var controls how long to train before SIGTERM (e.g., "24h", "48h", "7d")
CMD timeout --signal=TERM --foreground ${TRAIN_DURATION:-24h} ./trainer --fast

FROM debian:bookworm-slim   AS analyze
WORKDIR /app
EXPOSE 8888
COPY --from=analyze-builder     /app/target/release/analyze     /app/analyze
CMD ["./analyze"]

# Runtime stages for client side

FROM nginx:alpine           AS explore
EXPOSE 80
COPY --from=explore-builder     /app/dist                       /usr/share/nginx/html
CMD ["nginx", "-g", "daemon off;"]
