FROM rust:1.93 AS planner
WORKDIR /app
RUN cargo install cargo-chef
COPY --link Cargo.toml Cargo.toml
COPY --link test-env-compose/rootfs/repo/src/main.rs src/main.rs
RUN cargo chef prepare --recipe-path recipe.json

FROM rust:1.93 AS cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/home/root/app/target \
    cargo chef cook --release --recipe-path recipe.json

FROM rust:1.93 AS builder
WORKDIR /app
COPY --link --from=planner /app/ .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release

FROM rust:1.93-slim AS runtime
WORKDIR /app
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/api /usr/local/bin/

CMD ["/usr/local/bin/api"]
