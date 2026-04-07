# syntax=docker/dockerfile:1

# === Builder ===
FROM rust:1-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler libclang-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

ARG PACKAGE
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
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
