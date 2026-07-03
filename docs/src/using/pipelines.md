# Writing pipelines

> 🚧 *Chapter in progress.*

A pipeline is a directed acyclic graph (DAG) of nodes. Each node runs a command; edges are dependencies.

## Anatomy of a pipeline

<!-- name, projectId, nodes[]. -->

## Nodes: command, args, deps

<!-- One step = command + args + deps (node IDs). NodeId rules. -->

## The DAG rules

<!-- Must be acyclic; cycles rejected at creation. Parallel within a topo level. -->

## Worked example

<!-- Walk examples/pipeline-verbose.json: setup → build/test/lint → report. -->

## Workspaces

<!-- All nodes of a job share <root>/<job_id>; artifacts flow downstream. -->
