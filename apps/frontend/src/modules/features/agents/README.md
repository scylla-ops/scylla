# Agents

> [Scylla frontend](../../../../README.md) › `features/` › **agents** ·
> [agent guide](./AGENTS.md) · [architecture](../../../../docs/architecture.md)

Build agents are the machines that connect to Scylla, pick up jobs and run pipeline steps.
This module is where an organization sees which agents exist, whether they are alive, and how
their recent runs went.

## What it covers

- **The agent list** (`/:org/agents`) — one card per agent with its status and a run summary.
- **Agent details** (`/:org/agents/:agentId`) — statistics for a single agent: daily outcomes
  chart, live activity, logs.
- **Enrolling an agent** — creating one returns a token the operator pastes into the agent's
  configuration. That token is shown once.
- **The empty state** — `NoAgentsBanner` is the "you have no agents yet, here is how to add
  one" prompt. It is exported because the dashboard shows the same banner.

Everything is gated by `LIST_AGENTS`, declared once on the module and read by both the route
guard and the sidebar link.

## How it is built

The module follows the standard three-layer split.

**Domain** holds `AgentEntity` (the agent itself: id, name, status, last seen) and the structs
around it — `AgentStats` for the details page, `DailyOutcome` for the chart, `CreatedAgent` for
the one-time token returned at creation. `AgentsRepository` is the contract: list, get, stats,
create, delete. Nothing here knows gRPC exists.

**Infrastructure** implements that contract over gRPC-Web. Unlike the older modules, the data
source interface and its implementation live in one file — `AgentsRemoteDataSource` is simply an
alias of the repository interface, because there is exactly one transport and no coordination to
do. `GrpcAgentMapper` converts proto messages into the domain types.

**Presentation** is Svelte, as of Phase 3 of the migration. The three query hooks became
`agents.queries.ts`: `queryOptions` and `mutationOptions` objects, which is plain data with no
framework in it. `pipeline` and `dashboard` read agents through this module's barrel with the
same objects, and share one cache entry with the pages here.

The agents query decides its own `enabled` from `can(…)`. A caller builds the options inside
the `createQuery` callback, so the query starts when the permissions arrive.

The UI is split into small cards — `AgentCard`, `LiveNowCard`, `OutcomesChart`, `AgentLogs` — so
the pages stay layout-only, and the outcomes chart's bucket arithmetic sits in
`outcomes-chart.calculator.ts` where it can be tested without a DOM.

## Known gap: mocked panels

Part of this module is **not** wired to the backend yet. `agent-mock-data.ts` supplies the
live-jobs panel (`LiveNowCard`) and the log stream (`AgentLogs`), because no endpoint exists for
them today. The agent list, agent lookup and statistics are real.

Neither of those two components is mounted anywhere. The Phase 3 port carried them across as they
were rather than deleting them — deciding a component's fate is not a migration's call — so the
question is still open and now costs a rewrite to keep open.

If you are adding to this module, treat the mock as scaffolding to remove rather than an
abstraction to extend: the moment the backend exposes live jobs or agent logs, those components
should switch to real query factories and the file should disappear.

## Related modules

- [jobs](../jobs/README.md) — what agents actually execute.
- [dashboard](../dashboard/README.md) — reuses `NoAgentsBanner` and the outcomes chart shape.
- [apps](../apps/README.md) — the other machine identity in Scylla, for API clients rather than
  runners.
- [platform/authz](../../platform/authz/README.md) — the `LIST_AGENTS` gate.
