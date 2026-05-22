# syntax=docker/dockerfile:1.7

FROM node:22-alpine AS build

# Provide protoc on PATH so @protobuf-ts/protoc uses the system binary instead
# of downloading a release from GitHub at build time (which fails offline / on
# flaky networks with "socket hang up"). Our protos are proto3 and only import
# local files, so the system protoc needs no bundled well-known-types includes.
RUN apk add --no-cache protobuf

RUN corepack enable

ENV PNPM_HOME=/pnpm \
    PATH="/pnpm:/app/node_modules/.bin:$PATH"

WORKDIR /app

COPY apps/frontend/package.json apps/frontend/pnpm-lock.yaml apps/frontend/pnpm-workspace.yaml ./

RUN --mount=type=cache,id=pnpm-store,target=/pnpm/store \
    pnpm config set store-dir /pnpm/store && \
    pnpm install --frozen-lockfile

COPY crates/scylla-protocol/proto/ ../../crates/scylla-protocol/proto/
COPY apps/frontend/ .

ARG VITE_API_URL=""
ENV VITE_API_URL=$VITE_API_URL

RUN pnpm run build

FROM caddy:2-alpine

COPY apps/frontend/Caddyfile /etc/caddy/Caddyfile
COPY --from=build /app/dist /usr/share/caddy

EXPOSE 80

HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
    CMD wget -q --spider http://localhost/healthz || exit 1
