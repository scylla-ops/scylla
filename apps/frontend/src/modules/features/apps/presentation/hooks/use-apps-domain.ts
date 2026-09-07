import { useModuleDomain } from '@platform/di';
import type { AppsModule } from '../../apps.module.ts';

/** Typed access to the apps module's use cases. */
export const useAppsDomain = () => useModuleDomain<typeof AppsModule.domain>('apps');
