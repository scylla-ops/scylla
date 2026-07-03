# Backend (hexagonal)

> 🚧 *Chapter in progress.*

`scylla-core` follows a ports-and-adapters (hexagonal) layout. The domain is pure; I/O lives at the edges.

## Layers

<!-- domain/ (entities, value objects, errors) · application/ (use cases + ports) · infrastructure/ (adapters). -->

## Ports & adapters

<!-- Trait in application/ = port; Postgres/Argon2/Cedar impl in infrastructure/ = adapter. -->

## Crate composition

<!-- core (lib) → api (gRPC handlers lib) → control-plane (binary/composition root). -->

## Feature flags

<!-- scylla-core gates each domain behind a Cargo feature. -->
