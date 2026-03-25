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
    cargo build --release -p scylla-api --features grpc,rest && \
    cp target/release/scylla-api /app/scylla-api-bin

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

COPY --from=builder /app/scylla-api-bin ./scylla-api

RUN chown appuser:appuser ./scylla-api

ENV GIT_COMMIT=${GIT_COMMIT}

USER appuser

# gRPC: 50051, REST: 3000
EXPOSE 50051 3000

ENTRYPOINT ["./scylla-api"]
