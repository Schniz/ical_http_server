FROM alpine as chef
WORKDIR /app
RUN apk add --no-cache tzdata musl-dev openssl-dev curl bash gcc
ARG TARGETPLATFORM=linux/amd64
RUN case "${TARGETPLATFORM}" in \
  "linux/amd64")  RUST_TARGET=stable-x86_64-unknown-linux-musl  ;; \
  "linux/arm64")  RUST_TARGET=stable-aarch64-unknown-linux-gnu  ;; \
  "linux/arm/v7") RUST_TARGET=armv7-unknown-linux-musleabi  ;; \
  *) exit 1 ;; \
  esac; \
  curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain ${RUST_TARGET}
ENV PATH="/root/.cargo/bin:${PATH}"
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
FROM alpine AS runtime
WORKDIR /app
ENV RUST_LOG info
ENV PORT 8080
EXPOSE $PORT
COPY --from=builder /app/target/release/sensor_http /usr/local/bin
ENTRYPOINT ["/usr/local/bin/sensor_http"]