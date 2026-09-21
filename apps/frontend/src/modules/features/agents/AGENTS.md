# `features/agents` — agent guide

Build agents: the machines that pick up jobs, plus their run statistics.

**Layer** `features/` · **id** `agents` · **DI key** `agentsRepository`

## Import rules

- May import: `@platform/*` barrels, `@shared/*`, other features' `index.ts`.
- Must never import: `core/`, `layout/`, another feature's internals.
- Outside code reaches this module **only** through `index.ts`.

## Public API — `index.ts`

```typescript
type AgentEntity
type AgentStats, CreatedAgent, DailyOutcome
agentQueries, agentMutations
AGENTS_QUERY_KEY, AGENT_QUERY_KEY, AGENT_STATS_QUERY_KEY
NoAgentsBanner          // a .svelte component
```

Never add to this barrel: `agents.module.ts`, pages.

**This module is Svelte** (Phase 3). There is no `use-agents-domain.ts` and no hooks. The
three hooks became the factories in `presentation/agents.queries.ts`, which is what let
`pipeline` and `dashboard` keep consuming them while still in React: a `queryOptions` object
has no framework in it, so `useQuery` takes it unchanged and both halves share one cache entry.

**A React consumer must subscribe to the permissions store itself.** `agentQueries.byOrganization`
computes its own `enabled` from `can(LIST_AGENTS)`; in Svelte that is reactive, but a React
component only re-renders when something it subscribed to changes. Call
`useCan(Permission.LIST_AGENTS)` — `use-run-pipeline.ts` and `AgentOutcomesChart.tsx` both do —
or the query stays disabled for good once permissions arrive.

## Data contract

`AgentsRepository` — `domain/repository/agents.repository.ts`:

| Method | Returns |
|---|---|
| `listAgents(organizationId)` | `AgentEntity[]` |
| `getAgent(agentId)` | `AgentEntity` |
| `getAgentStats(agentId)` | `AgentStats` |
| `createAgent(organizationId, name)` | `CreatedAgent` |
| `deleteAgent(agentId)` | `void` |

All wrapped in `ScyllaResult`. Reach it from `agents.queries.ts` only — never from a component.

## Layout

```
agents.module.ts                     routes + nav + DI wiring (private; registry only)
index.ts                             public API
domain/
  entities/agent.entity.ts           AgentEntity
  structs/agent.struct.ts            AgentStats, CreatedAgent, DailyOutcome
  repository/agents.repository.ts    the contract
infrastructure/
  data/agents-remote.data-source.ts  interface + Impl, colocated
  data/grpc-agent.mapper.ts          GrpcAgentMapper
  repository/default-agents.repository.ts
presentation/
  agents.queries.ts                  every read and write, as query/mutation options
  ui/Agents.page.svelte, AgentDetails.page.svelte
  ui/agents.messages.ts              every string — `lingui extract` cannot read `.svelte`
  ui/components/                     AgentCard, AgentIdLink, AgentLogs, LiveNowCard,
                                     NoAgentsBanner, OutcomesChart  (all `.svelte`)
  ui/components/outcomes-chart.calculator.ts   the chart's arithmetic, testable without a DOM
  utils/agent-mock-data.ts           ⚠ placeholder data, see below
  utils/create-agent-form-items.ts   FormItem[] for the create dialog
```

## Routes & nav

| Mount | Path | Permission | Component |
|---|---|---|---|
| `organization` | `agents` (index) | `LIST_AGENTS` | `Agents.page.svelte` |
| `organization` | `agents/:agentId` | `READ_APP` | `AgentDetails.page.svelte` |

The detail page takes `agentId` as a **prop**: route params are the one thing a Svelte page
cannot read from a singleton, so `sveltePage` hands them down (`refacto_svelte.md` §6.7).

Sidebar: section `organization`, order `40`, icon `HardDriveIcon`, same permission.

## Rules that bite here

- **`agent-mock-data.ts` is placeholder data, not a fixture.** `AgentLogs` and `LiveNowCard`
  render from it because the backend has no endpoint for live jobs or agent logs yet. Do not
  build new UI on it — wire real repository calls, and delete the mock as soon as the endpoint
  lands.
- **`AgentLogs` and `LiveNowCard` have no consumer.** Nothing mounts them, and the Phase 3 port
  carried them across unchanged rather than deleting them — deciding a component's fate is not
  the migration's call. Same debt as `marketplace`'s list components from Phase 2. Delete them
  or wire them; do not leave the question open forever.
- **New strings go in `ui/agents.messages.ts`, never inside a `.svelte`.** Extraction does not
  read components, so a message declared there vanishes from the catalogs without failing a gate.
- The chart's arithmetic lives in `outcomes-chart.calculator.ts`, not in a `$derived.by` —
  bucket zero-filling has real edge cases (local vs UTC day, gaps, an empty window) and they are
  pinned in `@vitest-environment node`.
- The icon-only controls on `AgentCard` carry `sr-only` labels (`Agent actions`, `Delete agent`),
  which the React original lacked. A bits-ui menu sits in a floating layer that jsdom hides:
  reach its items with `findFloating` from `src/test/render.svelte.ts`, never `getByRole`.
- The data source here **colocates the interface and its impl** in one file
  (`AgentsRemoteDataSource` is a type alias of `AgentsRepository`), unlike the `jobs` /
  `pipeline` layout with `data-sources/` + `data/remote/`. Match the file you are in;
  do not "fix" one to the other in passing.
- `NoAgentsBanner` is exported because `jobs` shows it. It takes only `hasPendingJobs` and
  answers the rest — connectivity, permission — from the singletons. It is the one `.svelte`
  component this barrel exports; that is safe only because the shell imports
  `agents.module.ts` directly and never this file. **If the shell ever imports this barrel,
  export a loader instead** (see `organization`, Phase 2).

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
New strings: `pnpm extract && pnpm compile`.
