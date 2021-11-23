FROM rust as chef
RUN apt-get update && apt-get install tzdata -y
WORKDIR /app
RUN cargo install cargo-chef

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin sensor_http

# We do not need the Rust toolchain to run the binary!
FROM debian:buster-slim AS runtime
WORKDIR /app
RUN apt-get update && apt-get install -y libssl-dev ca-certificates
ENV RUST_LOG info
ENV PORT 8080
EXPOSE $PORT
COPY --from=builder /app/target/release/sensor_http /usr/local/bin
ENTRYPOINT ["/usr/local/bin/sensor_http"]