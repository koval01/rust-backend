FROM debian:trixie-slim

WORKDIR /app

RUN apt-get update && \
    apt-get install -y libssl3 libc6 ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

COPY docker-files/duolang /app/duolang

RUN chmod +x /app/duolang

ENV RUST_LOG=info

EXPOSE 8000

CMD ["./duolang"]

HEALTHCHECK --interval=10s --timeout=5s --start-period=30s --retries=5 \
  CMD curl -f http://localhost:8000/api/v1/health || exit 1
