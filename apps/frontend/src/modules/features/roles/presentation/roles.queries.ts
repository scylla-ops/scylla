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

// Resolved per call: tests swap the registry.
const domain = () => getModuleDomain<typeof RolesModule.domain>('roles');

export const ROLES_QUERY_KEY = ['permission-roles'] as const;

/** One prefix for every grant list, so one mutation invalidates them all. */
const GRANTS_QUERY_ROOT = 'permission-grants';

/** No scope: the system-wide list (system admins only). A scope narrows it. Different permissions, so different keys. */
export const GRANTS_QUERY_KEY = (scope?: PermissionScope, scopeId?: string) =>
  [GRANTS_QUERY_ROOT, scope ?? 'all', scopeId ?? ''] as const;

export const GRANTABLE_ROLES_QUERY_KEY = (scope?: PermissionScope) =>
  ['permission-grantable-roles', scope ?? 'all'] as const;

export const PERMISSION_VOCABULARY_QUERY_KEY = ['permission-vocabulary'] as const;

export const roleQueries = {
  /** Needs `MANAGE_ROLES`: without it, pass `enabled: false` and use `grantable`. */
  catalog: (options: { enabled?: boolean } = {}) =>
    queryOptions<RoleEntity[]>({
      queryKey: ROLES_QUERY_KEY,
      enabled: options.enabled ?? true,
      queryFn: async () => (await domain().permissionRepository.listRoles()).unwrap(),
    }),

  /** Needs no permission. Builtin roles only: merge with the catalog when it is readable. */
  grantable: (scope?: PermissionScope) =>
    queryOptions<GrantableRoleEntity[]>({
      queryKey: GRANTABLE_ROLES_QUERY_KEY(scope),
      queryFn: async () =>
        (await domain().permissionRepository.listGrantableRoles(scope)).unwrap(),
    }),

  /** System admins only. */
  allGrants: () =>
    queryOptions<GrantEntity[]>({
      queryKey: GRANTS_QUERY_KEY(),
      queryFn: async () => (await domain().permissionRepository.listGrants()).unwrap(),
    }),

  /** Needs `MANAGE_*_GRANTS` on the scope: pass `enabled: false` when the user lacks it. Idle while `scopeId` is `null`. */
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

  /** Every permission the backend knows, with its narrowest scope. New backend permissions show up without a frontend change. */
  vocabulary: () =>
    queryOptions<PermissionVocabularyEntity>({
      queryKey: PERMISSION_VOCABULARY_QUERY_KEY,
      queryFn: async () => (await domain().permissionRepository.listPermissionVocabulary()).unwrap(),
      staleTime: Infinity,
    }),
};

/** Every `can()` reads the store this writes. */
export const refreshMyPermissions = async (): Promise<void> => {
  const setPermissions = permissionsStore.getState().setPermissions;
  const userId = localStorage.getItem('userId') ?? '';

  if (userId === '') {
    setPermissions({ scopes: [] });
    return;
  }

  const result = await domain().permissionRepository.getMyPermissions();
  result.fold({
    onSuccess: permissions => setPermissions(permissions),
    // On failure: no permissions. The backend still enforces.
    onError: () => setPermissions({ scopes: [] }),
  });
};

/** Reloads only when user, organization or project change. */
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

/** For tests and sign-out. */
export const resetPermissionSync = (): void => {
  lastSyncedKey = null;
};

const invalidateRoles = () =>
  void getQueryClient().invalidateQueries({ queryKey: ROLES_QUERY_KEY });

/** `update` goes through `UpdateRoleUseCase`: read, apply `updateRole`, save. */
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

  /** A mutation: it runs when asked, and its answer is not cached. */
  effectivePermissions: () =>
    mutationOptions({
      mutationFn: async (principal: PrincipalEntity): Promise<EffectivePermissionsEntity> =>
        (await domain().permissionRepository.getEffectivePermissions(principal)).unwrap(),
    }),
};

/** Invalidates every grant list (views do not know each other), then reloads the caller's permissions. */
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

  /** Removes a principal from a scope entirely, unlike `revoke` (one grant). */
  revokeAllAccess: () =>
    mutationOptions({
      mutationFn: async (input: RevokeAllAccessInput) =>
        (await domain().permissionRepository.revokeAllAccess(input)).unwrap(),
      onSuccess: afterGrantChange,
    }),
};
