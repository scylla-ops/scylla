set dotenv-load
set quiet
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

DOCKER_USER  := env("DOCKER_USER", "godlyjaaaaj")
VERSION      := env("VERSION", "latest")
VITE_API_URL := env("VITE_API_URL", "http://localhost:50051")
DATABASE_URL := env("DATABASE_URL", "postgres://scylla:scylla@localhost:5432/scylla")
BUILDER      := env("BUILDER", "scylla-builder")

platforms := "linux/amd64,linux/arm64"

# -- Aliases --
alias u := up
alias s := start
alias d := down
alias l := logs

# Show available commands
default:
    @just --list

# -- Dev (local stack) --

# Build all services for local dev (native arch, PaaS edition)
[group('dev')]
local:
    docker compose build

# Build the SaaS edition of the control-plane (FEATURES=saas)
[group('dev')]
local-saas:
    docker compose -f docker-compose.yaml -f docker-compose.saas.yaml build

# Start the stack from already-present images (no pull, no rebuild)
[group('dev')]
[no-exit-message]
start:
    docker compose up -d

# Start the stack with the SaaS control-plane image
[group('dev')]
[no-exit-message]
start-saas:
    docker compose -f docker-compose.yaml -f docker-compose.saas.yaml up -d

# Pull latest images and start the stack
[group('dev')]
[no-exit-message]
up:
    docker compose pull
    docker compose up -d

# Stop all services
[group('dev')]
[no-exit-message]
down:
    docker compose down

# Show service logs (all or specific: just logs scylla-control-plane)
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

# -- Database (sqlx) --

# Start only the Postgres dev DB (for running migrations / tests locally)
[group('db')]
db-up:
    docker compose up -d postgres

# Apply pending migrations against $DATABASE_URL (uses sqlx-cli)
[group('db')]
db-migrate:
    DATABASE_URL={{DATABASE_URL}} cargo sqlx migrate run --source migrations

# Revert the most recent migration
[group('db')]
db-revert:
    DATABASE_URL={{DATABASE_URL}} cargo sqlx migrate revert --source migrations

# Regenerate the offline query cache (commit the resulting .sqlx/ dir)
[group('db')]
db-prepare:
    DATABASE_URL={{DATABASE_URL}} cargo sqlx prepare --workspace -- --tests

# Verify .sqlx/ is up-to-date
[group('db')]
db-prepare-check:
    DATABASE_URL={{DATABASE_URL}} cargo sqlx prepare --workspace --check -- --tests

# Drop & recreate the local Postgres dev volume (DESTRUCTIVE)
[group('db')]
[confirm("Drop scylla-postgres data volume?")]
db-reset:
    docker compose rm -sfv postgres
    docker volume rm scylla_postgres_data || true
    docker compose up -d postgres

# -- Release (manual multi-arch build & push to Docker Hub) --
#
# Plain buildx: the same Dockerfiles as local dev, built for amd64 + arm64 and
# pushed with their multi-arch manifest. The non-native platform builds under
# emulation, so a full release is slow — but it is one command, reproducible
# anywhere Docker runs, and needs no host toolchain.
#
#   just release-setup            # once per machine (+ docker login)
#   VERSION=0.3.0 just release    # SaaS stack: control-plane SaaS + agent + frontend

# One-time: create the multi-arch buildx builder
[group('release')]
release-setup:
    docker buildx inspect {{BUILDER}} >/dev/null 2>&1 || docker buildx create --name {{BUILDER}} --driver docker-container --bootstrap
    @echo "✓ buildx builder '{{BUILDER}}' ready (remember: docker login)"

# The PaaS control-plane is NOT part of `release` for now (use release-backend).
# Build & push the SaaS stack: control-plane SaaS + agent + frontend
[group('release')]
[no-exit-message]
release: release-saas (release-svc "scylla-agent") release-frontend

# Build & push the backend services (PaaS edition — not part of `release`)
[group('release')]
[no-exit-message]
release-backend: (release-svc "scylla-control-plane") (release-svc "scylla-agent")

# Build & push one backend service (e.g. just release-svc scylla-agent)
[group('release')]
[no-exit-message]
release-svc pkg: _info
    docker buildx build --builder {{BUILDER}} --platform {{platforms}} \
        --build-arg PACKAGE={{pkg}} \
        -t {{DOCKER_USER}}/{{pkg}}:{{VERSION}} \
        -t {{DOCKER_USER}}/{{pkg}}:latest \
        --push .

# Build & push the SaaS control-plane (tags :<version>-saas and :saas)
[group('release')]
[no-exit-message]
release-saas: _info
    docker buildx build --builder {{BUILDER}} --platform {{platforms}} \
        --build-arg PACKAGE=scylla-control-plane \
        --build-arg FEATURES=saas \
        -t {{DOCKER_USER}}/scylla-control-plane:{{VERSION}}-saas \
        -t {{DOCKER_USER}}/scylla-control-plane:saas \
        --push .

# Build & push the frontend (VITE_API_URL is baked into the assets)
[group('release')]
[no-exit-message]
release-frontend: _info
    docker buildx build --builder {{BUILDER}} --platform {{platforms}} \
        -f apps/frontend/Dockerfile \
        --build-arg VITE_API_URL={{VITE_API_URL}} \
        -t {{DOCKER_USER}}/scylla-frontend:{{VERSION}} \
        -t {{DOCKER_USER}}/scylla-frontend:latest \
        --push .

[private]
[no-exit-message]
_info:
    @echo "══ user={{DOCKER_USER}} version={{VERSION}} platforms={{platforms}} ══"
