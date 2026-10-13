# Introduction

**Scylla** is a distributed CI/CD platform.

## What is Scylla?

Scylla runs your pipelines across a fleet of machines from a single control
plane. Two binaries ship:

- **`scylla-ce`**: the control plane: the web UI, a gRPC API and its
  gRPC-Web translation, the inbound webhook ingress, and in-process job
  dispatch, all on one port.
- **`scylla-agent`**: a worker installed per machine, registered as an *App* and
  connected to the control plane over a persistent worker stream (there is no
  message broker).

A PostgreSQL database backs the control plane. The web UI is compiled into the control-plane binary and served from the same origin as the API, so there is nothing to deploy alongside it.

## How this book is organized

- **User Guide**: [Getting started](./getting-started.md) takes you from
  nothing to a pipeline that runs on your agent. [Architecture](./architecture.md),
  [Configuration](./configuration.md) and [Pipelines](./pipelines.md) follow.
- **Operations**: [Running an agent](./running-an-agent.md) and
  [Triggers](./triggers.md), with the GitHub webhook setup.
- **Reference**: the [Access model](./access-model.md) and the
  [Glossary](./glossary.md).

The `.proto` files in [scylla-protos] are the reference of the API. Their
comments give the exact rules of each call.

## Elsewhere

- [README] on GitHub
- [Full glossary][Glossary], with the terms of the code
- [Source on GitHub][repo]

[README]: https://github.com/scylla-ops/scylla#readme
[Glossary]: https://github.com/scylla-ops/scylla/blob/main/GLOSSARY.md
[repo]: https://github.com/scylla-ops/scylla
[scylla-protos]: https://github.com/scylla-ops/scylla-protos
