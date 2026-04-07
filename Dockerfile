# syntax=docker/dockerfile:1

# === Build: compile a single service from pre-cooked deps ===
ARG DEPS_IMAGE=scylla-deps:latest

FROM ${DEPS_IMAGE} AS build
WORKDIR /app
COPY . .
ARG PACKAGE
RUN cargo build --release -p ${PACKAGE}

# === Runtime: minimal Debian image ===
FROM debian:bookworm-slim AS runtime

LABEL org.opencontainers.image.source="https://github.com/scylla-ops/scylla"

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

RUN groupadd --gid 10001 appuser && \
    useradd --uid 10001 --gid appuser --no-create-home appuser

WORKDIR /app
USER appuser

ARG PACKAGE
COPY --from=build --chown=appuser:appuser /app/target/release/${PACKAGE} ./service
ENTRYPOINT ["./service"]
