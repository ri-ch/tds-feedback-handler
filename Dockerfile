# Runtime image
FROM alpine:3.20

LABEL org.opencontainers.image.title="tds-feedback-handler" \
      org.opencontainers.image.description="TDS feedback form handler"

# Run as "app" user
RUN addgroup -S appgroup && adduser -S appuser -G appgroup

USER appuser
WORKDIR /app

COPY ./target/x86_64-unknown-linux-musl/size-opt2/tds-feedback-handler /app/tds-feedback-handler

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget -qO- http://localhost:8080/status || exit 1

# Run the app
CMD ./tds-feedback-handler
