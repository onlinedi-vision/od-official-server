FROM alpine:3.22 AS builder
# trigger API_BUILD_AND_RUN
LABEL org.opencontainers.image.source=https://github.com/rust-lang/docker-rust
RUN apk add --no-cache \
        ca-certificates \
        gcc \
	curl \
	rust \
	cargo

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs 
COPY ./api ./api
RUN cd api && cargo build --release

ENTRYPOINT ["./api/target/release/api"]
