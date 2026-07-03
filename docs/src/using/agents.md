# Running an agent

> 🚧 *Chapter in progress.*

Agents are the workers that execute pipeline nodes. An agent is an App running the `scylla-agent` binary, connected to the control plane over a persistent worker stream.

## Create an App

<!-- In the UI: create App → get app id + one-time secret. -->

## Launch the agent

<!-- docker run or cargo run with --app-id / --app-secret / --control-plane-url. -->

## Agent options

<!-- CLI flags from scylla-agent/src/config.rs: workspace-root, reconnect, publish-buffer, keep-workspace. -->

## Presence & dispatch

<!-- Connected = open stream (no heartbeats). RunPipeline dispatches to connected authorized workers. -->
