/**
 * Build agents: the machines that pick up jobs, and their run statistics.
 *
 * The public API of the module. Anything outside `features/agents` imports from
 * here and nothing else — the internals are free to move. Deliberately excludes
 * `agents.module.ts`: the registry imports that directly so the barrel never
 * drags the module's wiring into another module's chunk.
 *
 * `useAgents` / `useAgent` / `useAgentStats` are gone (Phase 3). Their
 * replacements are options objects with no framework in them, so a React
 * consumer passes one straight to `useQuery` and shares the same cache entry as
 * the Svelte pages. **A React caller must subscribe to the permissions store
 * itself** — `useCan(Permission.LIST_AGENTS)` — or it will never re-render when
 * the permissions land and `enabled` will stay false for good.
 */
export type { AgentEntity } from './domain/entities/agent.entity.ts';
export type { AgentStats, CreatedAgent, DailyOutcome } from './domain/structs/agent.struct.ts';
export {
  agentQueries,
  agentMutations,
  AGENTS_QUERY_KEY,
  AGENT_QUERY_KEY,
  AGENT_STATS_QUERY_KEY,
} from './presentation/agents.queries.ts';
/**
 * The "no agents connected" banner the jobs page shows, behind a dynamic import.
 *
 * Re-exporting the component directly would drag the Svelte runtime and bits-ui
 * into every chunk that imports this barrel for something else — `dashboard` and
 * `pipeline` take `agentQueries` from here and are still React, and Rollup
 * cannot drop a component a barrel re-exports. A loader is a plain function:
 * tree-shakeable, and the chunk arrives when the banner is first rendered.
 * Same rule, and same remedy, as `organization`'s two dialogs.
 */
export const loadNoAgentsBanner = () =>
  import('./presentation/ui/components/NoAgentsBanner.svelte');
