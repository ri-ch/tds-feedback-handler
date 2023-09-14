# Runtime image
FROM alpine:latest

# Run as "app" user
RUN addgroup -S appgroup && adduser -S appuser -G appgroup

USER appuser
WORKDIR /app

COPY ./target/x86_64-unknown-linux-musl/size-opt2/tds-feedback-handler /app/tds-feedback-handler

# Run the app
CMD ./tds-feedback-handler