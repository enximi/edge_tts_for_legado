FROM rust:1-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY assets ./assets
COPY src ./src

RUN cargo build --locked --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/edge-tts-for-legado /usr/local/bin/edge-tts-for-legado

RUN mkdir -p /app/logs

EXPOSE 8000

CMD ["edge-tts-for-legado"]
