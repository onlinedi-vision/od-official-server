FROM alpine:3.22.1

LABEL org.opencontainers.image.source=https://github.com/rust-lang/docker-rust

RUN apk add --no-cache \
        ca-certificates \
        gcc \
	curl \
	rust \
	cargo

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs 

COPY ./ws ./ws
COPY ./api ./api
COPY ./cdn ./cdn
RUN touch /WSLOCK
RUN cd ws && cargo build --release
RUN cd api && cargo build --release
RUN mkdir /logs
CMD ["./ws/target/release/ws", ">", "/logs/WS_LOGS.logs", "2>", "/logs/ERROR_WS_LOGS.logs"]
CMD ["./api/target/release/api", ">", "/logs/API_LOGS.logs", "2>", "/logs/ERROR_API_LOGS.logs"]
