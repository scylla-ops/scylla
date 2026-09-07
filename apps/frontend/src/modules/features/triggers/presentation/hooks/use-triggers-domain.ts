import { useModuleDomain } from '@platform/di';
import type { TriggersModule } from '../../triggers.module.ts';

/** Typed access to the triggers module's use cases. */
export const useTriggersDomain = () => useModuleDomain<typeof TriggersModule.domain>('triggers');
