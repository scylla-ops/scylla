FROM rust:1.93.1-bookworm AS builder

ARG GIT_COMMIT=unknown

WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends protobuf-compiler && \
    rm -rf /var/lib/apt/lists/*

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release -p core && \
    cp target/release/core /app/core-bin

FROM debian:bookworm-slim AS runtime

ARG GIT_COMMIT=unknown

LABEL org.opencontainers.image.revision="${GIT_COMMIT}" \
      org.opencontainers.image.source="https://github.com/scylla-ops/scylla"

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --gid 1000 appuser && \
    useradd --uid 1000 --gid appuser --create-home appuser

WORKDIR /home/appuser

COPY --from=builder /app/core-bin ./core

RUN chown appuser:appuser ./core

ENV GIT_COMMIT=${GIT_COMMIT}

USER appuser

EXPOSE 8080

ENTRYPOINT ["./core"]
