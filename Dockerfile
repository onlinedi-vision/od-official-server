FROM alpine:3.22 AS builder

LABEL maintainer=kickhead13<ana.alexandru.gabriel@proton.me>

COPY . .
RUN apk add --no-cache ca-certificates gcc curl rust cargo \
&& curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
&& cargo build --release

ENTRYPOINT ["./target/release/api"]
