/**
 * Organizations: the top-level tenant, its members and the switcher in the shell.
 *
 * `organizationQueries` / `organizationMutations` replaced the hooks: they are
 * plain options objects, so the shell and the two features still on React run
 * them through react-query while this module's own Svelte UI runs them through
 * `createQuery` — one declaration, one cache entry.
 *
 * The two dialogs are reachable because the React shell still mounts them as
 * islands (`layout/context-selector/`), through a loader rather than directly —
 * see below. That ends in Phase 6.
 */
export type { OrganizationEntity } from './domain/entities/organization.entity.ts';
export {
  invalidateOrganizationMembers,
  organizationMutations,
  organizationQueries,
  MY_ORGANIZATIONS_QUERY_KEY,
  ORGANIZATIONS_QUERY_KEY,
  ORGANIZATION_MEMBERS_QUERY_KEY,
} from './presentation/organization.queries.ts';
export { createOrganizationItems } from './presentation/utils/create-organization-form-items.ts';
/**
 * The components that the shell shows, behind a dynamic import.
 *
 * The shell imports this barrel eagerly for the queries. A loader is a plain
 * function, so the component chunk loads only when the shell shows it.
 */
export const loadAddOrganizationDialog = () =>
  import('./presentation/ui/AddOrganizationDialog/AddOrganizationDialog.svelte');
export const loadOrganizationList = () => import('./presentation/ui/OrganizationList/OrganizationList.svelte');
