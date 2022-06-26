FROM rust:1.61.0-slim-bullseye AS builder

WORKDIR /app
COPY . .
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/app/crates/drop/migrations \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/rustup \
    set -eux; \
    rustup install stable; \
    cargo install sqlx-cli --no-default-features --features rustls,sqlite; \
    objcopy --compress-debug-sections /usr/local/cargo/bin/sqlx ./sqlx; \
    cargo build --release; \
    objcopy --compress-debug-sections target/release/drop ./drop

FROM debian:bullseye-slim

WORKDIR app

COPY ./Rocket.toml ./Rocket.toml
COPY --from=builder /app/crates/drop/migrations ./migrations
COPY --from=builder /app/sqlx ./sqlx
COPY --from=builder /app/start.sh ./start.sh
COPY --from=builder /app/drop ./drop
CMD ["./drop"]
