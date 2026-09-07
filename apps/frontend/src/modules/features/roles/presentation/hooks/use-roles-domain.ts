import { useModuleDomain } from '@platform/di';
import type { RolesModule } from '../../roles.module.ts';

/** Typed access to the roles module's use cases. */
export const useRolesDomain = () => useModuleDomain<typeof RolesModule.domain>('roles');
