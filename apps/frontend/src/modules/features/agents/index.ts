/**
 * Build agents: the machines that pick up jobs, and their run statistics.
 *
 * The public API of the module. Anything outside `features/agents` imports from
 * here and nothing else — the internals are free to move.
 * Deliberately excludes `agents.module.ts`: the registry imports that directly
 * so the barrel never drags the module's wiring into another module's chunk.
 */
export type { AgentEntity } from './domain/entities/agent.entity.ts';
export type { AgentStats, CreatedAgent, DailyOutcome } from './domain/structs/agent.struct.ts';
export { useAgents, useAgent, useAgentStats } from './presentation/hooks/use-agents.ts';
export { NoAgentsBanner } from './presentation/ui/components/NoAgentsBanner.tsx';
