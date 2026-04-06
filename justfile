set dotenv-load

platform := env("PLATFORM", "linux/amd64")
cache_repo := DOCKER_USER + "/scylla-cache"
services := "scylla-api scylla-broker scylla-agent scylla-recorder"

# Build for local dev (native arch, no push)
local:
    docker build -f Dockerfile.deps -t scylla-deps:latest .
    docker compose build

# Build & push deps image with registry cache
deps:
    docker buildx build \
        -f Dockerfile.deps \
        --platform {{platform}} \
        --cache-from type=registry,ref={{cache_repo}}:deps \
        --cache-to type=registry,ref={{cache_repo}}:deps,mode=max \
        -t {{DOCKER_USER}}/scylla-deps:{{VERSION}} \
        --push .

# Build & push a single service (e.g. just push-service scylla-api)
push-service svc: deps
    docker buildx build \
        --platform {{platform}} \
        --build-arg DEPS_IMAGE={{DOCKER_USER}}/scylla-deps:{{VERSION}} \
        --build-arg PACKAGE={{svc}} \
        --cache-from type=registry,ref={{cache_repo}}:{{svc}} \
        --cache-to type=registry,ref={{cache_repo}}:{{svc}},mode=max \
        -t {{DOCKER_USER}}/{{svc}}:{{VERSION}} \
        -t {{DOCKER_USER}}/{{svc}}:latest \
        --push .

# Build & push all services
push: deps
    #!/usr/bin/env bash
    for svc in {{services}}; do
        echo ""
        echo "══ Building $svc ══"
        docker buildx build \
            --platform {{platform}} \
            --build-arg DEPS_IMAGE={{DOCKER_USER}}/scylla-deps:{{VERSION}} \
            --build-arg PACKAGE=$svc \
            --cache-from type=registry,ref={{cache_repo}}:$svc \
            --cache-to type=registry,ref={{cache_repo}}:$svc,mode=max \
            -t {{DOCKER_USER}}/$svc:{{VERSION}} \
            -t {{DOCKER_USER}}/$svc:latest \
            --push .
    done
