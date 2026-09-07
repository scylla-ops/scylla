import { useModuleDomain } from '@platform/di/index.ts';
import type { AgentsModule } from '../../agents.module.ts';

/** Typed access to the agents module's use cases. */
export const useAgentsDomain = () => useModuleDomain<typeof AgentsModule.domain>('agents');
