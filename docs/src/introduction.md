# Introduction

**Scylla** is a distributed CI/CD platform.

> 🚧 **Documentation in progress.** This book is just getting started — most
> chapters are still being written and will land here soon. In the meantime, the
> [README] and [Glossary] on GitHub are the most complete references.

## What is Scylla?

Scylla runs your pipelines across a fleet of machines from a single control
plane. Two binaries ship:

- **`scylla-control-plane`** — the central brain: a gRPC API plus in-process job
  dispatch.
- **`scylla-agent`** — a worker installed per machine, registered as an *App* and
  connected to the control plane over a persistent worker stream (there is no
  message broker).

A PostgreSQL database backs the control plane, and a web UI drives everything.

## What's coming

The chapters in the sidebar are placeholders for now. Planned sections include:

- **Getting started** — run the full stack with Docker in one command.
- **Architecture** — control plane, agents/Apps, and the worker stream.
- **Configuration** — environment, ports, and database.
- **Operations** — running agents and wiring up triggers (cron & webhooks).
- **Reference** — the full glossary of Scylla terms.

## Until then

- [Quick start in the README][README]
- [Glossary of Scylla terms][Glossary]
- [Source on GitHub][repo]

[README]: https://github.com/scylla-ops/scylla#readme
[Glossary]: https://github.com/scylla-ops/scylla/blob/main/GLOSSARY.md
[repo]: https://github.com/scylla-ops/scylla
