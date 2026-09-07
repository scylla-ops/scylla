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
useAgents, useAgent, useAgentStats
NoAgentsBanner
```

Never add to this barrel: `agents.module.ts`, `use-agents-domain.ts`, pages.

## Data contract

`AgentsRepository` — `domain/repository/agents.repository.ts`:

| Method | Returns |
|---|---|
| `listAgents(organizationId)` | `AgentEntity[]` |
| `getAgent(agentId)` | `AgentEntity` |
| `getAgentStats(agentId)` | `AgentStats` |
| `createAgent(organizationId, name)` | `CreatedAgent` |
| `deleteAgent(agentId)` | `void` |

All wrapped in `ScyllaResult`. Reach it with `useAgentsDomain()` **inside a hook only** —
never from a component.

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
  hooks/use-agents-domain.ts         DI accessor (private)
  hooks/use-agents.ts                useAgents / useAgent / useAgentStats
  ui/Agents.page.tsx, AgentDetails.page.tsx
  ui/components/                     AgentCard, AgentIdLink, AgentLogs, LiveNowCard,
                                     NoAgentsBanner, OutcomesChart
  utils/agent-mock-data.ts           ⚠ placeholder data, see below
  utils/create-agent-form-items.ts   FormItem[] for the create dialog
```

## Routes & nav

| Mount | Path | Permission | Component |
|---|---|---|---|
| `organization` | `agents` (index) | `LIST_AGENTS` | `AgentsPage` |
| `organization` | `agents/:agentId` | `LIST_AGENTS` | `AgentDetailsPage` |

Sidebar: section `organization`, order `40`, icon `HardDriveIcon`, same permission.

## Rules that bite here

- **`agent-mock-data.ts` is placeholder data, not a fixture.** `Agents.page.tsx`
  (`mockCardStats`), `AgentLogs` and `LiveNowCard` still render from it because the backend
  has no endpoint for live jobs or agent logs yet. Do not build new UI on it — wire real
  repository calls, and delete the mock as soon as the endpoint lands.
- The data source here **colocates the interface and its impl** in one file
  (`AgentsRemoteDataSource` is a type alias of `AgentsRepository`), unlike the `jobs` /
  `pipeline` layout with `data-sources/` + `data/remote/`. Match the file you are in;
  do not "fix" one to the other in passing.
- `NoAgentsBanner` is exported because the dashboard shows it. Keep it prop-driven — it must
  not call `useAgentsDomain()` itself.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles` — all clean.
New strings: `pnpm extract && pnpm compile`.
