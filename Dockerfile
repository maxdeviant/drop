FROM rust:1.61.0-slim-bullseye AS builder

WORKDIR /app
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release; \
    objcopy --compress-debug-sections target/release/drop ./drop

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo install sqlx-cli --no-default-features --features rustls,sqlite; \
    objcopy --compress-debug-sections /usr/local/cargo/bin/sqlx ./sqlx

FROM debian:bullseye-slim

WORKDIR app

COPY ./Rocket.toml ./Rocket.toml
COPY ./start.sh ./start.sh
COPY ./crates/drop/migrations ./migrations
COPY --from=builder /app/sqlx ./sqlx
COPY --from=builder /app/drop ./drop
CMD ["./drop"]
