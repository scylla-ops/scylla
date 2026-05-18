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

# === Deps: cook the entire workspace's dependencies ===
# Independent of any service. CI builds this stage once per arch and
# pushes its layer cache to a shared registry tag (`scylla-deps:buildcache-*`),
# so the 4 service builds can pull pre-compiled deps instead of cooking them
# 4 times.
FROM chef AS deps
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json

# === Builder: build the chosen package on top of cooked deps ===
FROM deps AS builder
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
