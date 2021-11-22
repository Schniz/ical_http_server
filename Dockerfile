FROM rust as builder
WORKDIR /usr/src/myapp
COPY . .
RUN cargo install --path sensor_http

FROM debian:buster-slim
RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/myapp /usr/local/bin/sensor_http
CMD ["sensor_http"]
