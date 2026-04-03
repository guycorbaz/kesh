# --- Stage 1 : Build Rust ---
FROM rust:1.85-bookworm AS rust-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ crates/
RUN cargo build --release -p kesh-api \
    && strip /app/target/release/kesh-api

# --- Stage 2 : Build Frontend ---
FROM node:22-bookworm-slim AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ .
RUN npm run build

# --- Stage 3 : Image finale ---
FROM debian:bookworm-slim AS runtime
# curl nécessaire pour le healthcheck Docker
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=rust-builder /app/target/release/kesh-api ./kesh-api
COPY --from=frontend-builder /app/frontend/build ./static
ENV KESH_STATIC_DIR=/app/static
EXPOSE 3000
CMD ["./kesh-api"]
