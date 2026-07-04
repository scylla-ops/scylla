# Introduction

**Scylla** is a distributed CI/CD platform: pipelines run across a fleet of
machines from a single control plane. Two binaries ship:

- **`scylla-control-plane`** — the central brain: a gRPC API plus in-process
  job dispatch.
- **`scylla-agent`** — a worker installed per machine, registered as an *App*
  and connected to the control plane over a persistent worker stream (there is
  no message broker).

A PostgreSQL database backs the control plane, and a web UI drives everything.
The [system overview](./architecture/overview.md) has the full picture.

New here? Start with **[Getting started](./using/getting-started.md)** to bring
the stack up and run your first pipeline, then read
**[Core concepts](./using/concepts.md)** for the vocabulary.

- [Source on GitHub][repo]
- [Glossary of Scylla terms](./reference/glossary.md)

[repo]: https://github.com/scylla-ops/scylla
