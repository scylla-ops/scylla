# syntax=docker/dockerfile:1

# === Chef base ===
FROM rust:1-bookworm AS chef
# protobuf-compiler ships `protoc`; libprotobuf-dev ships the well-known-type
# .proto files under /usr/include/google/protobuf (e.g. timestamp.proto), which
# protoc auto-resolves. The protos import google/protobuf/timestamp.proto, so
# both are required — the compiler alone is not enough.
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler libprotobuf-dev && \
    rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --locked
WORKDIR /app

# === Planner: extract dependency recipe from Cargo.toml/Cargo.lock ===
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# === Deps: cook the dependencies the builder will reuse ===
# Only FEATURES gates the cook — NOT the package. Critically there is NO
# `ARG PACKAGE` in this stage: an in-scope ARG is folded into the RUN's cache key
# (BuildKit treats it as an implicit env prefix, even when the shell branch never
# expands it), which would give every service a distinct deps layer and silently
# un-share the PaaS cook.
# PaaS (FEATURES empty): cook the ENTIRE workspace once — service-independent, so
# the layer is SHARED across all PaaS service builds (control-plane, agent…); CI
# publishes it under one shared registry cache tag.
# SaaS (FEATURES set): cook the one feature-bearing crate (control-plane) with the
# exact features the builder uses below, so cook and build agree and the artifacts
# are reused. A mismatch (cook without the feature, build with it) makes cargo
# recompile every feature-gated dep.
FROM chef AS deps
COPY --from=planner /app/recipe.json recipe.json
ARG FEATURES=""
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    if [ -n "${FEATURES}" ]; then \
        cargo chef cook --release -p scylla-control-plane --features "${FEATURES}" --recipe-path recipe.json; \
    else \
        cargo chef cook --release --recipe-path recipe.json; \
    fi

# === Builder: build the chosen package on top of cooked deps ===
FROM deps AS builder
COPY . .

ARG PACKAGE
# Optional cargo features (e.g. FEATURES=saas for the SaaS edition). Empty = the
# default PaaS edition.
ARG FEATURES=""
ARG CARGO_BUILD_JOBS=2
ENV CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS}
# Use the committed .sqlx/ offline cache so query!/query_as! macros expand
# without needing a live Postgres at build time.
ENV SQLX_OFFLINE=true
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo build --release -p ${PACKAGE} ${FEATURES:+--features "${FEATURES}"} && \
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
