FROM rust:1.78-bookworm AS rust-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY test-data ./test-data
RUN cargo build -p neogenealogy

FROM node:20-bookworm AS web-builder
WORKDIR /app
COPY web ./web
RUN npm --prefix web ci && npm --prefix web run build

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=rust-builder /app/target/debug/neogenealogy /usr/local/bin/neogenealogy
COPY --from=web-builder /app/web/dist ./web/dist
COPY crates/storage/migrations ./migrations
COPY test-data ./test-data
EXPOSE 3000
ENV NEOGENEALOGY_HOST=0.0.0.0
ENV NEOGENEALOGY_PORT=3000
ENV NEOGENEALOGY_CORS_ORIGIN=*
VOLUME ["/data"]
CMD ["neogenealogy", "serve", "--db", "/data/neogenealogy.db", "--host", "0.0.0.0", "--port", "3000"]
