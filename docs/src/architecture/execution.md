# Pipeline execution

> 🚧 *Chapter in progress.*

How a `RunPipeline` becomes running processes on an agent.

## Dispatch

<!-- Control plane picks a connected authorized worker, sends JobDispatch. -->

## Topological execution

<!-- Kahn's algorithm over BTreeSet; parallel within a level; deterministic order. -->

## Node processes & workspaces

<!-- Each node = child process; shared per-job workspace; artifacts downstream. -->

## Status & log fan-out

<!-- NodeStarted/log lines/NodeCompleted streamed back; in-process fan-out, no recorder. -->

## Job & node states

<!-- JobStatus / NodeState machines; terminal states; Orphaned. -->
