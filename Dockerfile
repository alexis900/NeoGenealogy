FROM rust:1.78-bookworm AS builder
RUN apt-get update && apt-get install -y nodejs npm pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY web ./web
COPY test-data ./test-data
# Use debug build in Docker to avoid OOM on --release (fallback to release if enough RAM)
RUN cargo build -p neogenealogy || cargo build -p neogenealogy --release
RUN npm --prefix web ci && npm --prefix web run build

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/debug/neogenealogy /usr/local/bin/neogenealogy
COPY --from=builder /app/web/dist ./web/dist
COPY --from=builder /app/crates/storage/migrations ./migrations
COPY test-data ./test-data
EXPOSE 3000
ENV NEOGENEALOGY_HOST=0.0.0.0
ENV NEOGENEALOGY_PORT=3000
ENV NEOGENEALOGY_CORS_ORIGIN=*
VOLUME ["/data"]
CMD ["neogenealogy", "serve", "--db", "/data/neogenealogy.db", "--host", "0.0.0.0", "--port", "3000"]
