FROM alpine AS runtime
WORKDIR /app
ENV RUST_LOG info
ENV PORT 8080
EXPOSE $PORT
ARG TARGETPLATFORM
COPY ${TARGETPLATFORM} /usr/local/bin/
RUN chmod u+x /usr/local/bin/sensor_http
ENTRYPOINT ["/usr/local/bin/sensor_http"]