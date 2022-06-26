FROM rust:1.61.0-slim-bullseye AS builder

WORKDIR /app
COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/sqlx-target \
    set -eux; \
    cargo install --target-dir ./sqlx-target sqlx-cli --no-default-features --features rustls,sqlite; \
    objcopy --compress-debug-sections /usr/local/cargo/bin/sqlx ./sqlx

ENV DATABASE_URL=sqlite://drop.sqlite
RUN sqlx database setup --source ./crates/drop/migrations

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    set -eux; \
    cargo build --release; \
    objcopy --compress-debug-sections target/release/drop ./drop

FROM debian:bullseye-slim

WORKDIR app

COPY ./Rocket.toml ./Rocket.toml
COPY ./start.sh ./start.sh
COPY ./crates/drop/migrations ./migrations
COPY --from=builder /app/sqlx ./sqlx
COPY --from=builder /app/drop ./drop
CMD ["./drop"]
