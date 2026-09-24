import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { QueryClient } from '@tanstack/query-core';
import { setDependencyRegistry } from '@platform/di';
import { setQueryClient } from '@platform/query';
import {
  Permission,
  PermissionScope,
  PrincipalKind,
  permissionsStore,
} from '@platform/authz';
import { runMutationFn, runQueryFn } from '@/test/queries.ts';
import { ScyllaError, ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { RoleEntity } from '../../domain/entities/role.entity.ts';
import type { GrantEntity } from '../../domain/entities/grant.entity.ts';
import {
  GRANTABLE_ROLES_QUERY_KEY,
  GRANTS_QUERY_KEY,
  ROLES_QUERY_KEY,
  grantMutations,
  refreshMyPermissions,
  resetPermissionSync,
  roleMutations,
  roleQueries,
  syncMyPermissions,
} from '../roles.queries.ts';

const role: RoleEntity = {
  id: 'role-1',
  name: 'viewer',
  description: '',
  scope: PermissionScope.ORGANIZATION,
  origin: { kind: 'custom' },
  access: { kind: 'restricted', permissions: [Permission.READ_ORGANIZATION] },
};

const grant: GrantEntity = {
  id: 'grant-1',
  principal: { kind: PrincipalKind.USER, id: 'user-1' },
  roleId: 'role-1',
  scope: PermissionScope.ORGANIZATION,
  scopeId: 'org-1',
};

type Fn = ReturnType<typeof vi.fn>;

/**
 * The repository's methods as free handles.
 *
 * Asserting on `repository.listRoles` would pass an unbound method, which the
 * type-aware lint rejects — and rightly, since a mock read off the object is one
 * `this` away from a different function.
 */
let listRoles: Fn;
let getRole: Fn;
let createRole: Fn;
let updateRoleCall: Fn;
let deleteRole: Fn;
let listGrants: Fn;
let createGrant: Fn;
let revokeGrant: Fn;
let revokeAllAccess: Fn;
let listGrantableRoles: Fn;
let getEffectivePermissions: Fn;
let getMyPermissions: Fn;
let execute: Fn;
let queryClient: QueryClient;

beforeEach(() => {
  listRoles = vi.fn().mockResolvedValue(ScyllaResult.success([role]));
  getRole = vi.fn().mockResolvedValue(ScyllaResult.success(role));
  createRole = vi.fn().mockResolvedValue(ScyllaResult.success(role));
  updateRoleCall = vi.fn().mockResolvedValue(ScyllaResult.success(role));
  deleteRole = vi.fn().mockResolvedValue(ScyllaResult.success(undefined));
  listGrants = vi.fn().mockResolvedValue(ScyllaResult.success([grant]));
  createGrant = vi.fn().mockResolvedValue(ScyllaResult.success(grant));
  revokeGrant = vi.fn().mockResolvedValue(ScyllaResult.success(undefined));
  revokeAllAccess = vi.fn().mockResolvedValue(ScyllaResult.success(1));
  listGrantableRoles = vi.fn().mockResolvedValue(ScyllaResult.success([]));
  getEffectivePermissions = vi
    .fn()
    .mockResolvedValue(ScyllaResult.success({ permissions: [Permission.READ_ORGANIZATION] }));
  getMyPermissions = vi.fn().mockResolvedValue(
    ScyllaResult.success({
      scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }],
    }),
  );
  execute = vi.fn().mockResolvedValue(ScyllaResult.success(role));

  setDependencyRegistry({
    roles: {
      permissionRepository: {
        listRoles,
        getRole,
        createRole,
        updateRole: updateRoleCall,
        deleteRole,
        listGrants,
        createGrant,
        revokeGrant,
        revokeAllAccess,
        listGrantableRoles,
        listPermissionVocabulary: vi.fn().mockResolvedValue(ScyllaResult.success({ actions: [] })),
        getEffectivePermissions,
        getMyPermissions,
      },
      updateRole: { execute },
    },
  });

  queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  setQueryClient(queryClient);

  localStorage.setItem('userId', 'user-1');
  permissionsStore.setState({ permissions: null });
  resetPermissionSync();
});

afterEach(() => {
  setDependencyRegistry(null);
  setQueryClient(null);
  localStorage.clear();
  permissionsStore.setState({ permissions: null });
  resetPermissionSync();
});

describe('roleQueries', () => {
  it('reads the role catalog under the shared key', async () => {
    const options = roleQueries.catalog();

    expect(options.queryKey).toEqual(ROLES_QUERY_KEY);
    await expect(runQueryFn(options)).resolves.toEqual([role]);
  });

  it('lets a caller who cannot read the catalog opt out rather than collect a denial', () => {
    expect(roleQueries.catalog().enabled).toBe(true);
    expect(roleQueries.catalog({ enabled: false }).enabled).toBe(false);
  });

  it('keys grantable roles per scope — what you may hand out differs by scope', async () => {
    expect(GRANTABLE_ROLES_QUERY_KEY(PermissionScope.PROJECT)).not.toEqual(
      GRANTABLE_ROLES_QUERY_KEY(PermissionScope.ORGANIZATION),
    );

    await runQueryFn(roleQueries.grantable(PermissionScope.PROJECT));
    expect(listGrantableRoles).toHaveBeenCalledWith(PermissionScope.PROJECT);
  });

  it('separates the system-wide grant list from a scoped one', () => {
    expect(GRANTS_QUERY_KEY()).not.toEqual(
      GRANTS_QUERY_KEY(PermissionScope.ORGANIZATION, 'org-1'),
    );
  });

  it('asks for every grant when no scope is given', async () => {
    await expect(runQueryFn(roleQueries.allGrants())).resolves.toEqual([grant]);
    expect(listGrants).toHaveBeenCalledWith();
  });

  it('narrows a scoped grant list to its scope id', async () => {
    await runQueryFn(roleQueries.scopedGrants(PermissionScope.ORGANIZATION, 'org-1'));

    expect(listGrants).toHaveBeenCalledWith(PermissionScope.ORGANIZATION, 'org-1');
  });

  it('stays idle while the caller is still resolving which scope it is looking at', () => {
    expect(roleQueries.scopedGrants(PermissionScope.PROJECT, null).enabled).toBe(false);
    expect(roleQueries.scopedGrants(PermissionScope.PROJECT, 'project-1').enabled).toBe(true);
  });

  it("honours the caller's own gate even with a scope id in hand", () => {
    const options = roleQueries.scopedGrants(PermissionScope.PROJECT, 'project-1', {
      enabled: false,
    });

    expect(options.enabled).toBe(false);
  });

  it('never restales the vocabulary — it is a code-owned catalog, not tenant data', () => {
    expect(roleQueries.vocabulary().staleTime).toBe(Infinity);
  });
});

describe('roleMutations', () => {
  it('creates a role through the repository and refreshes the catalog', async () => {
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries');
    const options = roleMutations.create();

    await runMutationFn(options, {
      name: 'viewer',
      description: '',
      scope: PermissionScope.ORGANIZATION,
      access: { kind: 'fullControl' },
    });
    expect(createRole).toHaveBeenCalled();

    options.onSuccess?.(role, {} as never, undefined, undefined as never);
    expect(invalidate).toHaveBeenCalledWith({ queryKey: ROLES_QUERY_KEY });
  });

  it('routes an update through the use case, not straight to the repository', async () => {
    await runMutationFn(roleMutations.update(), {
      id: 'role-1',
      name: 'viewer',
      description: '',
      access: { kind: 'fullControl' },
    });

    expect(execute).toHaveBeenCalled();
    expect(updateRoleCall).not.toHaveBeenCalled();
  });

  it('deletes by role id', async () => {
    await runMutationFn(roleMutations.remove(), 'role-1');

    expect(deleteRole).toHaveBeenCalledWith('role-1');
  });

  it('reports effective permissions on demand, without caching them', async () => {
    const principal = { kind: PrincipalKind.USER, id: 'user-1' };

    await runMutationFn(roleMutations.effectivePermissions(), principal);

    expect(getEffectivePermissions).toHaveBeenCalledWith(principal);
  });
});

describe('grantMutations', () => {
  /** Every grant list, whatever its scope — one prefix, one invalidation. */
  const GRANT_PREFIX = [GRANTS_QUERY_KEY()[0]];

  it('creates a grant and invalidates every grant list, not just the one in view', async () => {
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries');
    const options = grantMutations.create();

    await runMutationFn(options, {
      principal: { kind: PrincipalKind.USER, id: 'user-1' },
      roleId: 'role-1',
      scope: PermissionScope.ORGANIZATION,
      scopeId: 'org-1',
    });
    expect(createGrant).toHaveBeenCalled();

    options.onSuccess?.(grant, {} as never, undefined, undefined as never);
    expect(invalidate).toHaveBeenCalledWith({ queryKey: GRANT_PREFIX });
  });

  it("reloads the caller's own permissions after a grant change", () => {
    grantMutations.create().onSuccess?.(grant, {} as never, undefined, undefined as never);

    // The store is written asynchronously; what matters is that the call went out.
    expect(getMyPermissions).toHaveBeenCalled();
  });

  it('revokes one grant by id', async () => {
    await runMutationFn(grantMutations.revoke(), 'grant-1');

    expect(revokeGrant).toHaveBeenCalledWith('grant-1');
  });

  it('clears a principal from a scope entirely, which revoking one grant does not', async () => {
    const input = {
      principal: { kind: PrincipalKind.USER, id: 'user-1' },
      scope: PermissionScope.ORGANIZATION,
      scopeId: 'org-1',
    };

    await runMutationFn(grantMutations.revokeAllAccess(), input);

    expect(revokeAllAccess).toHaveBeenCalledWith(input);
  });
});

describe('refreshMyPermissions', () => {
  it('fills the store with what the backend answers', async () => {
    await refreshMyPermissions();

    expect(permissionsStore.getState().permissions?.scopes).toHaveLength(1);
  });

  it('settles a signed-out session as "no permissions" rather than leaving it loading', async () => {
    localStorage.removeItem('userId');

    await refreshMyPermissions();

    expect(permissionsStore.getState().permissions).toEqual({ scopes: [] });
    expect(getMyPermissions).not.toHaveBeenCalled();
  });

  it('settles a failed lookup the same way — the backend stays the real enforcer', async () => {
    getMyPermissions.mockResolvedValue(ScyllaResult.error(new ScyllaError('nope')));

    await refreshMyPermissions();

    expect(permissionsStore.getState().permissions).toEqual({ scopes: [] });
  });
});

describe('syncMyPermissions', () => {
  /** The call is fired without being awaited; let the microtasks drain. */
  const settle = () => new Promise(resolve => setTimeout(resolve, 0));

  it('loads once for a session key and never again on a re-render', async () => {
    syncMyPermissions('org-1', null);
    syncMyPermissions('org-1', null);
    await settle();

    expect(getMyPermissions).toHaveBeenCalledTimes(1);
  });

  it('reloads when the organization changes — the answer depends on it', async () => {
    syncMyPermissions('org-1', null);
    syncMyPermissions('org-2', null);
    await settle();

    expect(getMyPermissions).toHaveBeenCalledTimes(2);
  });

  it('reloads when the project changes within the same organization', async () => {
    syncMyPermissions('org-1', 'project-1');
    syncMyPermissions('org-1', 'project-2');
    await settle();

    expect(getMyPermissions).toHaveBeenCalledTimes(2);
  });

  it('reloads after a sign-out, because the user is part of the key', async () => {
    syncMyPermissions('org-1', null);
    localStorage.setItem('userId', 'user-2');
    syncMyPermissions('org-1', null);
    await settle();

    expect(getMyPermissions).toHaveBeenCalledTimes(2);
  });
});
