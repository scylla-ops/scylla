import { useModuleDomain } from '@platform/di/index.ts';
import type { JobsModule } from '../../jobs.module.ts';

/** Typed access to the jobs module's use cases. */
export const useJobsDomain = () => useModuleDomain<typeof JobsModule.domain>('jobs');
