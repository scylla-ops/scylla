import { getQueryClient, mutationOptions, queryOptions } from '@platform/query';
import { getModuleDomain } from '@platform/di';
import {
  permissionsStore,
  type EffectivePermissionsEntity,
  type PermissionScope,
  type PrincipalEntity,
} from '@platform/authz';
import type { RoleCreationData, RoleEntity } from '../domain/entities/role.entity.ts';
import type { GrantEntity } from '../domain/entities/grant.entity.ts';
import type { PermissionVocabularyEntity } from '../domain/entities/permission-vocabulary.entity.ts';
import type {
  CreateGrantInput,
  RevokeAllAccessInput,
} from '../domain/repository/permission.repository.ts';
import type { UpdateRoleInput } from '../domain/use-cases/update-role.use-case.ts';
import type { GrantableRoleEntity } from '../domain/entities/grantable-role.entity.ts';
import type { RolesModule } from '../roles.module.ts';

/**
 * Every read and write this module performs, declared as plain data.
 *
 * This is what the thirteen hooks under `presentation/hooks/` were, with the
 * framework taken out. Four of them cross a barrel — `membership` went Svelte in
 * Phase 3 and reads the same role catalog and the same grants — and a
 * `queryOptions` object has no framework in it, so `useQuery` and `createQuery`
 * take the same one and share a cache entry. Two declarations of one resource
 * is exactly how a cache forks into two entries that disagree.
 *
 * The repository is resolved per call, never at module load: the registry is
 * installed by the composition root and swapped by tests.
 */
const domain = () => getModuleDomain<typeof RolesModule.domain>('roles');

export const ROLES_QUERY_KEY = ['permission-roles'] as const;

/** Prefix shared by every grant list, so one mutation invalidates them all. */
const GRANTS_QUERY_ROOT = 'permission-grants';

/**
 * `undefined` scope is the system-wide list, which only a system administrator
 * may read; a scope narrows it to one organization or project, readable by an
 * administrator of that scope. The two are different requests with different
 * permissions, so they are different keys.
 */
export const GRANTS_QUERY_KEY = (scope?: PermissionScope, scopeId?: string) =>
  [GRANTS_QUERY_ROOT, scope ?? 'all', scopeId ?? ''] as const;

export const GRANTABLE_ROLES_QUERY_KEY = (scope?: PermissionScope) =>
  ['permission-grantable-roles', scope ?? 'all'] as const;

export const PERMISSION_VOCABULARY_QUERY_KEY = ['permission-vocabulary'] as const;

export const roleQueries = {
  /**
   * The dynamic role catalog.
   *
   * The backend gates it behind `MANAGE_ROLES`, so a caller that only
   * administers one organization or project cannot read it and passes
   * `enabled: false` rather than asking for a denial — working from
   * {@link roleQueries.grantable} instead.
   */
  catalog: (options: { enabled?: boolean } = {}) =>
    queryOptions<RoleEntity[]>({
      queryKey: ROLES_QUERY_KEY,
      enabled: options.enabled ?? true,
      queryFn: async () => (await domain().permissionRepository.listRoles()).unwrap(),
    }),

  /**
   * The roles that may be handed out at `scope`.
   *
   * Needs no permission at all — it is a compile-time constant on the backend,
   * not tenant data — which makes it the only role list an organization or
   * project administrator can read. Builtins only; custom roles are grantable
   * but absent, so a view that can also read the catalog merges the two.
   */
  grantable: (scope?: PermissionScope) =>
    queryOptions<GrantableRoleEntity[]>({
      queryKey: GRANTABLE_ROLES_QUERY_KEY(scope),
      queryFn: async () =>
        (await domain().permissionRepository.listGrantableRoles(scope)).unwrap(),
    }),

  /** Every grant in the installation — refused to anyone but a system admin. */
  allGrants: () =>
    queryOptions<GrantEntity[]>({
      queryKey: GRANTS_QUERY_KEY(),
      queryFn: async () => (await domain().permissionRepository.listGrants()).unwrap(),
    }),

  /**
   * The grants bound to one organization or project — what a tenant
   * administrator may read, and the backing of the member views.
   *
   * `scopeId` may be `null` while the caller is still resolving which scope it
   * is looking at; the query simply stays idle. `enabled` is the caller's own
   * gate: the backend answers this only to a holder of `MANAGE_*_GRANTS` on the
   * scope, so a view that already knows the user lacks it should not ask and
   * collect a denial toast for an answer it never needed.
   */
  scopedGrants: (
    scope: PermissionScope,
    scopeId: string | null,
    options: { enabled?: boolean } = {},
  ) =>
    queryOptions<GrantEntity[]>({
      queryKey: GRANTS_QUERY_KEY(scope, scopeId ?? ''),
      enabled: (options.enabled ?? true) && !!scopeId,
      queryFn: async () =>
        (await domain().permissionRepository.listGrants(scope, scopeId ?? '')).unwrap(),
    }),

  /**
   * The permission vocabulary — every permission the backend knows, with the
   * narrowest scope at which it means anything.
   *
   * A closed, code-owned catalog rather than tenant data, so it never goes
   * stale. It is what makes the role editor's tree grow a new backend
   * permission without a frontend change.
   */
  vocabulary: () =>
    queryOptions<PermissionVocabularyEntity>({
      queryKey: PERMISSION_VOCABULARY_QUERY_KEY,
      queryFn: async () => (await domain().permissionRepository.listPermissionVocabulary()).unwrap(),
      staleTime: Infinity,
    }),
};

/**
 * Reloads the signed-in user's effective permissions into the store.
 *
 * The `useCallback` in `use-refresh-my-permissions.ts` with React taken out, so
 * a grant mutation can call it from either side. Writing the store is the whole
 * point: every `can()` in the app reads it.
 */
export const refreshMyPermissions = async (): Promise<void> => {
  const setPermissions = permissionsStore.getState().setPermissions;
  const userId = localStorage.getItem('userId') ?? '';

  if (userId === '') {
    // Not signed in — settled with no permissions rather than stuck loading.
    setPermissions({ scopes: [] });
    return;
  }

  const result = await domain().permissionRepository.getMyPermissions();
  result.fold({
    onSuccess: permissions => setPermissions(permissions),
    // Failed lookup → settled as "no permissions": gated UI explains the denial
    // and the backend stays the real enforcer.
    onError: () => setPermissions({ scopes: [] }),
  });
};

/**
 * The signed-in user's permissions, reloaded only when the session key changes.
 *
 * The key is user + organization + project: once at login, then on a context
 * switch, and never on a re-render. `usePermissionSync` was a React effect
 * holding this guard in a ref; the guard is what the shell actually needed, and
 * it has no framework in it. The shell still owns *when* to call this — Phase 6
 * turns that side into a Svelte effect and this function does not change.
 */
let lastSyncedKey: string | null = null;

export const syncMyPermissions = (
  organizationId: string | null,
  projectId: string | null,
): void => {
  const userId = localStorage.getItem('userId') ?? '';
  const key = `${userId}/${organizationId ?? ''}/${projectId ?? ''}`;
  if (lastSyncedKey === key) return; // nothing relevant changed

  lastSyncedKey = key;
  void refreshMyPermissions();
};

/** Forgets the sync key, so the next call reloads. For tests and sign-out. */
export const resetPermissionSync = (): void => {
  lastSyncedKey = null;
};

const invalidateRoles = () =>
  void getQueryClient().invalidateQueries({ queryKey: ROLES_QUERY_KEY });

/**
 * The role catalog's writes.
 *
 * `update` goes through `UpdateRoleUseCase` — the codebase's only use case —
 * because it reads the role, applies the pure `updateRole` entity function and
 * saves the result: three steps a repository method cannot do. `create` and
 * `delete` call the repository, because that is all they are.
 */
export const roleMutations = {
  create: () =>
    mutationOptions({
      mutationFn: async (input: RoleCreationData) =>
        (await domain().permissionRepository.createRole(input)).unwrap(),
      onSuccess: invalidateRoles,
    }),

  update: () =>
    mutationOptions({
      mutationFn: async (input: UpdateRoleInput) =>
        (await domain().updateRole.execute(input)).unwrap(),
      onSuccess: invalidateRoles,
    }),

  remove: () =>
    mutationOptions({
      mutationFn: async (roleId: string) =>
        (await domain().permissionRepository.deleteRole(roleId)).unwrap(),
      onSuccess: invalidateRoles,
    }),

  /**
   * "What can this principal actually do?", on demand.
   *
   * A mutation rather than a query because it runs when the reader asks, not on
   * mount, and its answer is a one-off report rather than cached state.
   */
  effectivePermissions: () =>
    mutationOptions({
      mutationFn: async (principal: PrincipalEntity): Promise<EffectivePermissionsEntity> =>
        (await domain().permissionRepository.getEffectivePermissions(principal)).unwrap(),
    }),
};

/**
 * Every grant mutation invalidates the whole grant prefix: a grant created from
 * the project view also changes the organization's list, and neither view knows
 * the other exists. It then reloads the caller's own permissions, because a
 * grant change may alter what they are allowed to see.
 */
const afterGrantChange = () => {
  void getQueryClient().invalidateQueries({ queryKey: [GRANTS_QUERY_ROOT] });
  void refreshMyPermissions();
};

export const grantMutations = {
  create: () =>
    mutationOptions({
      mutationFn: async (input: CreateGrantInput) =>
        (await domain().permissionRepository.createGrant(input)).unwrap(),
      onSuccess: afterGrantChange,
    }),

  revoke: () =>
    mutationOptions({
      mutationFn: async (grantId: string) =>
        (await domain().permissionRepository.revokeGrant(grantId)).unwrap(),
      onSuccess: afterGrantChange,
    }),

  /**
   * Clears a principal's grants at a scope and beneath it. Distinct from
   * {@link grantMutations.revoke}, which drops one grant by id: this is the
   * "remove them from here entirely" operation, and the only one that leaves no
   * inert project-scoped grant behind.
   */
  revokeAllAccess: () =>
    mutationOptions({
      mutationFn: async (input: RevokeAllAccessInput) =>
        (await domain().permissionRepository.revokeAllAccess(input)).unwrap(),
      onSuccess: afterGrantChange,
    }),
};
