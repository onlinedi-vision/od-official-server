FROM clux/muslrust:1.93.1-stable AS base 
WORKDIR /app
COPY --link --from=bare-repo . .
COPY --link Cargo* .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release --target x86_64-unknown-linux-musl

FROM base AS builder
COPY --link src/ src/
RUN touch src/main.rs
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release --target x86_64-unknown-linux-musl

FROM scratch AS runtime
COPY --link --from=builder /app/target/x86_64-unknown-linux-musl/release/api .
CMD ["/api"]
