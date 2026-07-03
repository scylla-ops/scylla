# System overview

> 🚧 *Chapter in progress.*

The big picture: one control plane, many agents, a persistent worker stream, no message broker.

## Components

<!-- control-plane (gRPC + dispatch + webhook ingress), agents, postgres, frontend. -->

## The worker stream

<!-- Agents open a WorkerService stream; dispatch + status/logs flow over it in-process. -->

## Request lifecycle

<!-- UI → gRPC-Web → control plane → use case → repo/Cedar → dispatch → agent. -->

<!-- TODO: mermaid diagram of control plane + agents. -->
