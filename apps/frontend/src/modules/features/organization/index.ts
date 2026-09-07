/**
 * Organizations: the top-level tenant, its members and the switcher in the shell.
 *
 * The public API of the module. The shell (`layout`) builds the organization
 * switcher from `OrganizationList` and `AddOrganizationDialog`, so those two
 * components are part of the contract rather than internals.
 */
export type { OrganizationEntity } from './domain/entities/organization.entity.ts';
export { useOrganizations } from './presentation/hooks/useOrganizations.ts';
export { useCreateOrganization } from './presentation/hooks/useCreateOrganization.ts';
export {
  useOrganizationMembers,
  ORGANIZATION_MEMBERS_QUERY_KEY,
} from './presentation/hooks/use-organization-members.ts';
export { createOrganizationItems } from './presentation/utils/create-organization-form-items.ts';
export { OrganizationList } from './presentation/ui/OrganizationList.tsx';
export { AddOrganizationDialog } from './presentation/ui/AddOrganizationDialog.tsx';
