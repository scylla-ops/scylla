# syntax=docker/dockerfile:1

# === Chef base ===
FROM rust:1-bookworm AS chef
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler libclang-dev && \
    rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --locked
WORKDIR /app

# === Planner: extract dependency recipe from Cargo.toml/Cargo.lock ===
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# === Builder: cook deps once, then build the chosen package ===
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json

COPY . .

ARG PACKAGE
ARG CARGO_BUILD_JOBS=2
ENV CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS}
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo build --release -p ${PACKAGE} && \
    cp target/release/${PACKAGE} /app/service

# === Runtime ===
FROM debian:bookworm-slim AS runtime

LABEL org.opencontainers.image.source="https://github.com/scylla-ops/scylla"

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

RUN groupadd --gid 10001 appuser && \
    useradd --uid 10001 --gid appuser --no-create-home appuser

WORKDIR /app
USER appuser

COPY --from=builder --chown=appuser:appuser /app/service ./service
ENTRYPOINT ["./service"]
