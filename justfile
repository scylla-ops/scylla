set dotenv-load

DOCKER_USER := env_var("DOCKER_USER")
VERSION     := env_var("VERSION")
platform    := env("PLATFORM", "linux/amd64")
cache_repo  := DOCKER_USER + "/scylla-cache"

# Show available commands
default:
    @just --list

# Build deps + services for local dev (native arch)
[group('dev')]
local:
    docker build -f Dockerfile.deps -t scylla-deps:latest .
    docker compose build

# Start all services
[group('dev')]
up:
    docker compose up

# Stop all services
[group('dev')]
down:
    docker compose down

# Build & push deps image with registry cache
[group('registry')]
deps:
    docker buildx build \
        -f Dockerfile.deps \
        --platform {{platform}} \
        --cache-from type=registry,ref={{cache_repo}}:deps \
        --cache-to type=registry,ref={{cache_repo}}:deps,mode=max \
        -t {{DOCKER_USER}}/scylla-deps:{{VERSION}} \
        --push .

# Build & push a single service (e.g. just push scylla-api)
[group('registry')]
push svc: deps (_build-push svc)

# Build & push all services
[group('registry')]
push-all: deps (_build-push "scylla-api") (_build-push "scylla-broker") (_build-push "scylla-agent") (_build-push "scylla-recorder")

[private]
_build-push svc:
    @echo "══ Building {{svc}} ══"
    docker buildx build \
        --platform {{platform}} \
        --build-arg DEPS_IMAGE={{DOCKER_USER}}/scylla-deps:{{VERSION}} \
        --build-arg PACKAGE={{svc}} \
        --cache-from type=registry,ref={{cache_repo}}:{{svc}} \
        --cache-to type=registry,ref={{cache_repo}}:{{svc}},mode=max \
        -t {{DOCKER_USER}}/{{svc}}:{{VERSION}} \
        -t {{DOCKER_USER}}/{{svc}}:latest \
        --push .
