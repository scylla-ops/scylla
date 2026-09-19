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
 * The two dialogs the shell opens, behind a dynamic import.
 *
 * Re-exporting the components directly would put the Svelte runtime and bits-ui
 * in the entry chunk — `layout` imports this barrel eagerly for the queries, and
 * Rollup cannot drop a component it re-exports. A loader is a plain function:
 * tree-shakeable, and the chunk arrives when the dialog is first opened.
 * `LazySvelteIsland` (`@shared`) is the consumer side.
 */
export const loadAddOrganizationDialog = () =>
  import('./presentation/ui/AddOrganizationDialog.svelte');
export const loadEditOrganizationDialog = () =>
  import('./presentation/ui/EditOrganizationDialog.svelte');
