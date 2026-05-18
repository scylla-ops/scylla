set dotenv-load
set quiet
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

DOCKER_USER := env("DOCKER_USER", "godlyjaaaaj")
VERSION     := env("VERSION", "latest")
cache_repo  := DOCKER_USER + "/scylla-cache"

platform := env("PLATFORM", "linux/amd64")

# -- Aliases --
alias u := up
alias s := start
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

# Start the stack from already-present images (no pull, no rebuild)
[group('dev')]
[no-exit-message]
start:
    docker compose up -d

# -- Database (sqlx) --

DATABASE_URL := env("DATABASE_URL", "postgres://scylla:scylla@localhost:5432/scylla")

# Start only the Postgres dev DB (for running migrations / tests locally)
[group('db')]
db-up:
    docker compose up -d postgres

# Tail Postgres logs
[group('db')]
db-logs:
    docker compose logs -f postgres

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

# Verify .sqlx/ is up-to-date (used in CI)
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
