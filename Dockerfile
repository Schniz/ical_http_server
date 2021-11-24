FROM alpine as selector
ARG TARGETPLATFORM
COPY releases .
RUN case "${TARGETPLATFORM}"; in \
  "linux/arm/v7") \
  BINARY_PATH="releases/armv7/sensor_http" \
  ;; \
  "linux/arm64") \
  BINARY_PATH="releases/arm64/sensor_http" \
  ;; \
  "linux/amd64") \
  BINARY_PATH="releases/amd64/sensor_http" \
  ;; \
  *) \
  exit 1 \
  ;; \
  esac; \
  cp $BINARY_PATH /app

FROM alpine AS runtime
WORKDIR /app
ENV RUST_LOG info
ENV PORT 8080
EXPOSE $PORT
COPY --from=selector /app /usr/local/bin/
RUN chmod u+x /app
ENTRYPOINT ["/app"]