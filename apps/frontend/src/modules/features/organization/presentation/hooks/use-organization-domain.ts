import { useModuleDomain } from '@platform/di';
import type { OrganizationModule } from '../../organization.module.ts';

/** Typed access to the organization module's use cases. */
export const useOrganizationDomain = () => useModuleDomain<typeof OrganizationModule.domain>('organization');
