# The web app, served by Caddy, which also forwards the API to the server.
# Built from the repository root, with the server's address baked in:
#   docker build -f apps/web/Dockerfile --build-arg RANDOMTASK_SERVER=https://api.randomtask.ch .
# Trixie, for a clang (19) new enough for SQLite's C23 `[[noreturn]]`;
# bookworm's clang 14 rejects it.
FROM rust:1.97-slim-trixie AS wasm
RUN apt-get update && apt-get install -y --no-install-recommends clang \
    && rm -rf /var/lib/apt/lists/*
RUN rustup target add wasm32-unknown-unknown \
    && cargo install wasm-bindgen-cli --version 0.2.129 --locked
WORKDIR /src
COPY crates/client crates/client
RUN cargo build --release --locked --target wasm32-unknown-unknown \
        --manifest-path crates/client/web/Cargo.toml \
    && wasm-bindgen --target web --out-dir /wasm \
        crates/client/web/target/wasm32-unknown-unknown/release/randomtask_web.wasm

FROM node:26-bookworm-slim AS site
ARG RANDOMTASK_SERVER=https://api.randomtask.ch
WORKDIR /src
COPY package.json package-lock.json ./
COPY packages packages
COPY apps/web apps/web
COPY apps/desktop/package.json apps/desktop/package.json
RUN npm ci --workspace @randomtask/web --include-workspace-root
COPY --from=wasm /wasm apps/web/src/wasm
RUN cd apps/web && VITE_RANDOMTASK_SERVER=$RANDOMTASK_SERVER npx vite build

FROM caddy:2-alpine
LABEL org.opencontainers.image.source=https://github.com/davidswords/randomtask
COPY deploy/Caddyfile /etc/caddy/Caddyfile
COPY --from=site /src/apps/web/dist /srv
