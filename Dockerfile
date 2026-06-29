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
# curl nécessaire pour le healthcheck Docker + le téléchargement pdfium ci-dessous.
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

# pdfium (Story 12-5b — rendu des pages PDF en image pour le décodage QR à
# l'import, DC1-bis). Binaire natif libpdfium.so depuis une release ÉPINGLÉE de
# bblanchon/pdfium-binaries (PAS `latest`), checksum SHA-256 vérifié AVANT
# extraction. Placé dans /usr/local/lib (chemin ldconfig standard, pas de
# LD_LIBRARY_PATH) — c'est le nom attendu par pdfium-render
# (`Pdfium::bind_to_system_library`). amd64 uniquement (L3 — cohérent release.yml).
# Licence pdfium : Apache-2.0 / BSD-3-Clause (https://github.com/bblanchon/pdfium-binaries).
ARG PDFIUM_RELEASE=chromium/7920
ARG PDFIUM_SHA256=49ab3afbd4e6c1e284b5f2898129c8bb8a10fd785c1c5392c8c1fc70242f9ced
RUN set -eux; \
    url="https://github.com/bblanchon/pdfium-binaries/releases/download/$(echo "$PDFIUM_RELEASE" | sed 's:/:%2F:')/pdfium-linux-x64.tgz"; \
    curl -fsSL -o /tmp/pdfium.tgz "$url"; \
    echo "${PDFIUM_SHA256}  /tmp/pdfium.tgz" | sha256sum -c -; \
    tar -xzf /tmp/pdfium.tgz -C /tmp lib/libpdfium.so; \
    mv /tmp/lib/libpdfium.so /usr/local/lib/libpdfium.so; \
    rm -rf /tmp/pdfium.tgz /tmp/lib; \
    ldconfig

WORKDIR /app
COPY --from=rust-builder /app/target/release/kesh-api ./kesh-api
COPY --from=frontend-builder /app/frontend/build ./static
COPY crates/kesh-i18n/locales ./locales
ENV KESH_STATIC_DIR=/app/static
ENV KESH_LOCALES_DIR=/app/locales
EXPOSE 80
CMD ["./kesh-api"]
