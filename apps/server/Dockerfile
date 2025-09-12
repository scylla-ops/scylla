# syntax=docker/dockerfile:1
ARG RUST_VERSION=1.89.0
ARG APP_NAME=scylla-core

FROM rust:${RUST_VERSION}-alpine AS build
ARG APP_NAME
WORKDIR /app

# Install host build dependencies.
RUN apk add --no-cache clang lld musl-dev git pkgconfig openssl-dev make perl bash gcc protoc

COPY apps/server apps/server
COPY libs libs

WORKDIR apps/server

RUN --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/apps/server/target \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
cargo build --locked --release && \
cp ./target/release/$APP_NAME /bin/$APP_NAME

FROM alpine:3.18 AS final

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

COPY --from=build /bin/$APP_NAME /bin/

EXPOSE 50051

ENTRYPOINT ["/bin/scylla-core"]
