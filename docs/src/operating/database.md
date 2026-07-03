# Database & migrations

> 🚧 *Chapter in progress.*

Scylla stores everything in PostgreSQL. Schema is versioned SQL applied at boot.

## Schema & migrations

<!-- migrations/*.sql applied via sqlx::migrate! at boot (run_migrations). -->

## The offline query cache

<!-- .sqlx/ lets Docker builds compile without a live DB; `just db-prepare`. -->

## Local database workflow

<!-- just db-up / db-migrate / db-revert / db-reset. -->

## Upgrades

<!-- PG18 volume layout keeps pg_upgrade --link viable. -->
