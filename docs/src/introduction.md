# Introduction

**Scylla** is a distributed CI/CD platform.

## What is Scylla?

Scylla runs your pipelines across a fleet of machines from a single control
plane. Two binaries ship:

- **`scylla-control-plane`** — the central brain: a gRPC API plus in-process job
  dispatch.
- **`scylla-agent`** — a worker installed per machine, registered as an *App* and
  connected to the control plane over a persistent worker stream (there is no
  message broker).

A PostgreSQL database backs the control plane, and a web UI drives everything. The
[system overview](./architecture/overview.md) has the full picture.

## How this book is organized

- **[Using Scylla](./using/getting-started.md)** — for anyone running pipelines:
  getting started, core concepts, writing pipelines, agents, secrets, triggers,
  and access.
- **[Operating Scylla](./operating/deployment.md)** — for whoever deploys it:
  deployment, configuration, database, security, and troubleshooting.
- **[Architecture](./architecture/overview.md)** — how it works inside: the
  hexagonal backend, the authorization model, pipeline execution, and the gRPC
  protocol.
- **[Contributing](./contributing/development.md)** — local development, project
  layout, and conventions.
- **[Reference](./reference/configuration.md)** — the full configuration reference
  and the glossary.

## New here?

Start with **[Getting started](./using/getting-started.md)** to bring the stack up
and run your first pipeline, then read **[Core concepts](./using/concepts.md)** for
the vocabulary.

- [Source on GitHub][repo]
- [Glossary of Scylla terms](./reference/glossary.md)

[repo]: https://github.com/scylla-ops/scylla
