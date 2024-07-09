# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.79.0
ARG APP_NAME=api

FROM rust:${RUST_VERSION}-slim AS build
ARG APP_NAME

RUN apt-get update && \
    apt-get install -y git clang lld musl-tools pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=migrations,target=migrations \
    --mount=type=bind,source=.sqlx,target=.sqlx \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release && \
    cp ./target/release/$APP_NAME /bin/server

FROM debian:bookworm-slim AS final

ARG UID=10001
RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates wkhtmltopdf && \
    rm -rf /var/lib/apt/lists/*

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

COPY --from=build /bin/server /bin/
COPY src/data src/data

EXPOSE 3000

CMD ["/bin/server"]
