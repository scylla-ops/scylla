import { useModuleDomain } from '@platform/di/index.ts';
import type { ProjectModule } from '../../project.module.ts';

/** Typed access to the project module's use cases. */
export const useProjectDomain = () => useModuleDomain<typeof ProjectModule.domain>('project');
