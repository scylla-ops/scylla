set dotenv-load
set quiet
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

DOCKER_USER := env("DOCKER_USER", "godlyjaaaaj")
VERSION     := env("VERSION", "latest")
cache_repo  := DOCKER_USER + "/scylla-cache"

platform := env("PLATFORM", "linux/amd64")

# -- Aliases --
alias u := up
alias d := down
alias l := logs

# Show available commands
default:
    @just --list

# -- Dev --

# Build all services for local dev (native arch)
[group('dev')]
local:
    docker compose build

# Start all services (pulls latest images, runs detached)
[group('dev')]
[no-exit-message]
up:
    docker compose pull
    docker compose up -d

# Pull latest images without (re)starting containers
[group('dev')]
[no-exit-message]
pull:
    docker compose pull

# Refresh a running stack: pull latest images and recreate containers
[group('dev')]
[no-exit-message]
update:
    docker compose pull
    docker compose up -d

# Stop all services
[group('dev')]
[no-exit-message]
down:
    docker compose down

# Show service logs (all or specific: just logs scylla-api)
[group('dev')]
[no-exit-message]
logs *svc:
    docker compose logs -f {{svc}}

# Show running containers
[group('dev')]
[no-exit-message]
status:
    docker compose ps

# Remove this project's containers, networks, volumes, and locally-built images
[group('dev')]
[confirm("Remove Scylla containers, networks, volumes, and local images?")]
clean:
    docker compose down --rmi local --volumes --remove-orphans

# -- Registry --

# Build & push a single service (e.g. just push scylla-api)
[group('registry')]
[no-exit-message]
push svc: (_build-push svc)

# Build & push all services
[group('registry')]
[no-exit-message]
push-all: (_info) (_build-push "scylla-api") (_build-push "scylla-broker") (_build-push "scylla-agent") (_build-push "scylla-recorder")

[private]
[no-exit-message]
_info:
    @echo "══ user={{DOCKER_USER}} version={{VERSION}} platform={{platform}} ══"

[private]
[no-exit-message]
_build-push svc:
    @echo "══ Building {{svc}} ══"
    docker buildx build \
        --platform {{platform}} \
        --build-arg PACKAGE={{svc}} \
        --cache-from type=registry,ref={{cache_repo}}:{{svc}} \
        --cache-to type=registry,ref={{cache_repo}}:{{svc}},mode=max \
        -t {{DOCKER_USER}}/{{svc}}:{{VERSION}} \
        -t {{DOCKER_USER}}/{{svc}}:latest \
        --push .
