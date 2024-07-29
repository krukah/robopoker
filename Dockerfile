FROM rust:1.80 as builder
WORKDIR /usr/src/robopoker
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/robopoker/target/release/robopoker /usr/local/bin/robopoker
CMD ["robopoker"]