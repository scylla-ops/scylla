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
# Critically there is NO `ARG PACKAGE` in this stage: an in-scope ARG is folded
# into the RUN's cache key (BuildKit treats it as an implicit env prefix), which
# would give every service a distinct deps layer and silently un-share the cook.
# Cook the ENTIRE workspace once — service-independent, so the layer is SHARED
# across all service builds (control-plane, agent…).
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
# Use the committed .sqlx/ offline cache so query!/query_as! macros expand
# without needing a live Postgres at build time.
ENV SQLX_OFFLINE=true
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

# The agent's default --workspace-root lives under /var/lib/scylla. Bake the
# directory in with the right owner so a named volume mounted there inherits
# appuser ownership (Docker copies ownership from the image path on first
# mount) — without it the agent gets a root-owned dir and fails on permission.
RUN groupadd --gid 10001 appuser && \
    useradd --uid 10001 --gid appuser --no-create-home appuser && \
    mkdir -p /var/lib/scylla/workspaces && \
    chown -R appuser:appuser /var/lib/scylla

WORKDIR /app
USER appuser

COPY --from=builder --chown=appuser:appuser /app/service ./service
ENTRYPOINT ["./service"]
