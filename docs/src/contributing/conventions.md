# Conventions

> 🚧 *Chapter in progress.*

The patterns to follow when adding code.

## Domain modeling

<!-- Entities vs value objects; fallible constructors enforcing invariants. -->

## Use cases & ports

<!-- One use-case struct per aggregate; depend on Arc<dyn Port>. -->

## Errors

<!-- DomainError / DomainResult in core; mapped to gRPC status in scylla-api handlers. -->

## Adding a repository

<!-- Trait in application/ + Pg…Repository in infrastructure/postgres/; sqlx query!/query_as!. -->

## Naming & lints

<!-- Authz seven-word vocabulary; role naming <scope>-<role>; workspace clippy config. -->
