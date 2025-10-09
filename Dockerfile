FROM alpine:3.22 AS builder


LABEL maintainer=kickhead13<ana.alexandru.gabriel@proton.me>
RUN apk add --no-cache \
        ca-certificates \
        gcc \
	curl \
	rust \
	cargo

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs 
COPY . .
RUN cargo build --release

ENTRYPOINT ["./target/release/api"]
