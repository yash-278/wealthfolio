# syntax=docker/dockerfile:1
FROM node:24.11.0-alpine AS frontend-deps
WORKDIR /app
RUN npm install -g pnpm@10.33.4
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY apps/frontend/package.json apps/frontend/package.json
COPY packages/ui/package.json packages/ui/package.json
COPY packages/addon-sdk/package.json packages/addon-sdk/package.json
COPY packages/addon-dev-tools/package.json packages/addon-dev-tools/package.json
ENV CI=1
RUN pnpm install --frozen-lockfile

FROM frontend-deps AS frontend
COPY tsconfig*.json ./
COPY packages ./packages
COPY apps/frontend ./apps/frontend
COPY apps/tauri/tauri.conf.json apps/tauri/tauri.conf.json
COPY apps/server/src/api.rs apps/server/src/api.rs
ARG CONNECT_AUTH_URL=
ARG CONNECT_AUTH_PUBLISHABLE_KEY=
ENV CONNECT_AUTH_URL=${CONNECT_AUTH_URL} CONNECT_AUTH_PUBLISHABLE_KEY=${CONNECT_AUTH_PUBLISHABLE_KEY} BUILD_TARGET=web
RUN pnpm --filter frontend... build && mv dist /web-dist

FROM rust:1.95-alpine AS chef
WORKDIR /app
RUN apk add --no-cache clang lld build-base git pkgconfig openssl-dev openssl-libs-static sqlite-dev
RUN cargo install cargo-chef --version 0.1.73 --locked
ENV OPENSSL_STATIC=1 CARGO_BUILD_JOBS=2
RUN mkdir -p .cargo && printf '[profile.release.package."*"]\nopt-level=1\n' > .cargo/config.toml
ARG RELEASE_OPT_LEVEL=0
ENV CARGO_PROFILE_RELEASE_OPT_LEVEL=${RELEASE_OPT_LEVEL}

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY apps/server ./apps/server
COPY apps/tauri/Cargo.toml apps/tauri/Cargo.toml
RUN mkdir -p apps/tauri/src && printf 'fn main(){}' > apps/tauri/src/main.rs && touch apps/tauri/src/lib.rs
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS backend-deps
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --package wealthfolio-server

FROM backend-deps AS backend
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY apps/server ./apps/server
ARG CONNECT_AUTH_URL=
ARG CONNECT_AUTH_PUBLISHABLE_KEY=
ENV CONNECT_AUTH_URL=${CONNECT_AUTH_URL} CONNECT_AUTH_PUBLISHABLE_KEY=${CONNECT_AUTH_PUBLISHABLE_KEY}
RUN cargo build --locked --release --package wealthfolio-server

FROM alpine:3.19 AS runtime
WORKDIR /app
COPY --from=backend /app/target/release/wealthfolio-server /usr/local/bin/wealthfolio-server
COPY --from=frontend /web-dist ./dist
ENV WF_DB_PATH=/data/wealthfolio.db
RUN addgroup -S -g 1000 wealthfolio && adduser -S -u 1000 -G wealthfolio -H -s /sbin/nologin wealthfolio && mkdir -p /data && chown wealthfolio:wealthfolio /data
USER 1000:1000
EXPOSE 8088
CMD ["/usr/local/bin/wealthfolio-server"]
