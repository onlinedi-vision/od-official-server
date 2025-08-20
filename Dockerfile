FROM alpine:3.22.1

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
RUN mkdir /logs
CMD ["./api/target/release/api", ">", "/logs/API_LOGS.logs", "2>", "/logs/ERROR_API_LOGS.logs", "&"]
