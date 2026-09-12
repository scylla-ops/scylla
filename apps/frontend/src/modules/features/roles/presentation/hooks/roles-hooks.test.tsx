import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { ReactNode } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { renderHook, waitFor } from '@testing-library/react';
import { DependenciesProvider } from '@platform/di';
import { usePermissionsStore, PermissionScope, PrincipalKind } from '@platform/authz';
import { ScyllaResult, ScyllaError } from '@shared/utils/scylla-result.ts';
import { useGrantableRoles, GRANTABLE_ROLES_QUERY_KEY } from './use-grantable-roles';
import { useGrants, useScopedGrants, GRANTS_QUERY_KEY } from './use-grants';
import { useRoles } from './use-roles';
import { useRefreshMyPermissions } from './use-refresh-my-permissions';
import { usePermissionVocabulary } from './use-permission-vocabulary';
import { useEffectivePermissions } from './use-effective-permissions';
import { UpdateRoleUseCase } from '@/modules/features/roles/domain/use-cases/update-role.use-case.ts';
import type { PermissionRepository } from '@/modules/features/roles/domain/repository/permission.repository.ts';
import type { RoleEntity } from '@/modules/features/roles/domain/entities/role.entity.ts';
import type { GrantEntity } from '@/modules/features/roles/domain/entities/grant.entity.ts';
import type { GrantableRoleEntity } from '@/modules/features/roles/domain/entities/grantable-role.entity.ts';

const role = (overrides: Partial<RoleEntity> = {}): RoleEntity => ({
  id: 'role-1',
  name: 'CI runner',
  description: 'runs pipelines',
  scope: PermissionScope.PROJECT,
  origin: { kind: 'custom' },
  access: { kind: 'fullControl' },
  ...overrides,
});

const grant = (overrides: Partial<GrantEntity> = {}): GrantEntity => ({
  id: 'grant-1',
  principal: { kind: PrincipalKind.USER, id: 'user-1' },
  roleId: 'role-1',
  scope: PermissionScope.PROJECT,
  scopeId: 'project-1',
  ...overrides,
});

const grantable = (overrides: Partial<GrantableRoleEntity> = {}): GrantableRoleEntity => ({
  roleId: 'organization-admin',
  scope: PermissionScope.ORGANIZATION,
  kind: 0,
  description: 'Full control of the organization',
  ...overrides,
});

const makeFakeRepository = (overrides: Partial<PermissionRepository> = {}) => {
  const listRoles = overrides.listRoles ?? vi.fn().mockResolvedValue(ScyllaResult.success([role()]));
  const getRole = overrides.getRole ?? vi.fn().mockResolvedValue(ScyllaResult.success(role()));
  const createRole = overrides.createRole ?? vi.fn().mockResolvedValue(ScyllaResult.success(role()));
  const updateRole = overrides.updateRole ?? vi.fn().mockResolvedValue(ScyllaResult.success(role()));
  const deleteRole = overrides.deleteRole ?? vi.fn().mockResolvedValue(ScyllaResult.success(undefined));
  const getEffectivePermissions =
    overrides.getEffectivePermissions ??
    vi.fn().mockResolvedValue(ScyllaResult.success({ scopes: [] }));
  const getMyPermissions =
    overrides.getMyPermissions ?? vi.fn().mockResolvedValue(ScyllaResult.success({ scopes: [] }));
  const listGrants = overrides.listGrants ?? vi.fn().mockResolvedValue(ScyllaResult.success([grant()]));
  const createGrant = overrides.createGrant ?? vi.fn().mockResolvedValue(ScyllaResult.success(grant()));
  const revokeGrant = overrides.revokeGrant ?? vi.fn().mockResolvedValue(ScyllaResult.success(undefined));
  const revokeAllAccess = overrides.revokeAllAccess ?? vi.fn().mockResolvedValue(ScyllaResult.success(2));
  const listGrantableRoles =
    overrides.listGrantableRoles ?? vi.fn().mockResolvedValue(ScyllaResult.success([grantable()]));
  const listPermissionVocabulary =
    overrides.listPermissionVocabulary ??
    vi.fn().mockResolvedValue(ScyllaResult.success({ actions: [] }));

  const repository: PermissionRepository = {
    listRoles,
    getRole,
    createRole,
    updateRole,
    deleteRole,
    getEffectivePermissions,
    getMyPermissions,
    listGrants,
    createGrant,
    revokeGrant,
    revokeAllAccess,
    listGrantableRoles,
    listPermissionVocabulary,
  };
  return {
    repository,
    listRoles,
    getRole,
    createRole,
    updateRole,
    deleteRole,
    getEffectivePermissions,
    getMyPermissions,
    listGrants,
    createGrant,
    revokeGrant,
    revokeAllAccess,
    listGrantableRoles,
    listPermissionVocabulary,
  };
};

const wrapperFor = (repository: PermissionRepository) => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const Wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      <DependenciesProvider
        registry={{ roles: { permissionRepository: repository, updateRole: new UpdateRoleUseCase(repository) } }}
      >
        {children}
      </DependenciesProvider>
    </QueryClientProvider>
  );
  return { Wrapper, queryClient };
};

beforeEach(() => {
  usePermissionsStore.setState({ permissions: null });
  localStorage.clear();
});

describe('useGrantableRoles', () => {
  it('lists the roles grantable at a scope', async () => {
    const { repository, listGrantableRoles } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useGrantableRoles(PermissionScope.ORGANIZATION), {
      wrapper: Wrapper,
    });

    await waitFor(() => expect(result.current.grantableRoles).toHaveLength(1));
    expect(listGrantableRoles).toHaveBeenCalledWith(PermissionScope.ORGANIZATION);
  });

  it('the query key distinguishes "all" (no scope) from an actual scope', () => {
    expect(GRANTABLE_ROLES_QUERY_KEY()).toEqual(['permission-grantable-roles', 'all']);
    expect(GRANTABLE_ROLES_QUERY_KEY(PermissionScope.PROJECT)).toEqual([
      'permission-grantable-roles',
      PermissionScope.PROJECT,
    ]);
  });
});

describe('useRoles', () => {
  it('lists the dynamic role catalog', async () => {
    const { repository } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useRoles(), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.roles).toHaveLength(1));
  });

  it('is enabled by default but respects enabled:false for a caller without MANAGE_ROLES', () => {
    const { repository, listRoles } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    renderHook(() => useRoles({ enabled: false }), { wrapper: Wrapper });
    expect(listRoles).not.toHaveBeenCalled();
  });

  it('createRole/deleteRole invalidate the roles list', async () => {
    const { repository, createRole, deleteRole } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useRoles(), { wrapper: Wrapper });

    await result.current.createRole.mutateAsync({
      name: 'new-role',
      description: 'd',
      scope: PermissionScope.PROJECT,
      access: { kind: 'fullControl' },
    });
    expect(createRole).toHaveBeenCalled();
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['permission-roles'] });

    invalidateSpy.mockClear();
    await result.current.deleteRole.mutateAsync('role-1');
    expect(deleteRole).toHaveBeenCalledWith('role-1');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['permission-roles'] });
  });

  it('updateRole goes through the real UpdateRoleUseCase (read-merge-save), not a direct repository call', async () => {
    const { repository, getRole, updateRole } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useRoles(), { wrapper: Wrapper });

    await result.current.updateRole.mutateAsync({ id: 'role-1', name: 'Renamed' });

    expect(getRole).toHaveBeenCalledWith('role-1');
    expect(updateRole).toHaveBeenCalledWith(expect.objectContaining({ name: 'Renamed' }));
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['permission-roles'] });
  });
});

describe('useGrants (system-wide)', () => {
  it('lists every grant in the installation (no scope)', async () => {
    const { repository, listGrants } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useGrants(), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.grants).toHaveLength(1));
    expect(listGrants).toHaveBeenCalledWith();
  });

  it('createGrant invalidates the whole grants prefix and refreshes my permissions', async () => {
    localStorage.setItem('userId', 'user-1'); // refreshMyPermissions no-ops when signed out
    const { repository, createGrant, getMyPermissions } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useGrants(), { wrapper: Wrapper });

    await result.current.createGrant.mutateAsync({
      principal: { kind: PrincipalKind.USER, id: 'user-2' },
      roleId: 'role-1',
      scope: PermissionScope.PROJECT,
      scopeId: 'project-1',
    });

    expect(createGrant).toHaveBeenCalled();
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['permission-grants'] });
    expect(getMyPermissions).toHaveBeenCalled();
  });
});

describe('useScopedGrants', () => {
  it('lists grants scoped to one organization/project', async () => {
    const { repository, listGrants } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(
      () => useScopedGrants(PermissionScope.PROJECT, 'project-1'),
      { wrapper: Wrapper },
    );

    await waitFor(() => expect(result.current.grants).toHaveLength(1));
    expect(listGrants).toHaveBeenCalledWith(PermissionScope.PROJECT, 'project-1');
  });

  it('does not fetch while scopeId is still resolving (null)', () => {
    const { repository, listGrants } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    renderHook(() => useScopedGrants(PermissionScope.PROJECT, null), { wrapper: Wrapper });
    expect(listGrants).not.toHaveBeenCalled();
  });

  it('does not fetch when the caller disables it (no MANAGE_*_GRANTS)', () => {
    const { repository, listGrants } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    renderHook(() => useScopedGrants(PermissionScope.PROJECT, 'project-1', { enabled: false }), {
      wrapper: Wrapper,
    });
    expect(listGrants).not.toHaveBeenCalled();
  });

  it('system-wide and scoped grants use different cache keys (never collide)', () => {
    expect(GRANTS_QUERY_KEY()).not.toEqual(GRANTS_QUERY_KEY(PermissionScope.PROJECT, 'project-1'));
  });

  it('revokeAllAccess invalidates the grants prefix too', async () => {
    const { repository, revokeAllAccess } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(
      () => useScopedGrants(PermissionScope.PROJECT, 'project-1'),
      { wrapper: Wrapper },
    );

    const revoked = await result.current.revokeAllAccess.mutateAsync({
      principal: { kind: PrincipalKind.USER, id: 'user-1' },
      scope: PermissionScope.PROJECT,
      scopeId: 'project-1',
    });

    expect(revokeAllAccess).toHaveBeenCalled();
    expect(revoked).toBe(2);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['permission-grants'] });
  });
});

describe('useRefreshMyPermissions', () => {
  it('fetches and stores the effective permissions for the signed-in user', async () => {
    localStorage.setItem('userId', 'user-1');
    const permissions = { scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' as const } }] };
    const { repository, getMyPermissions } = makeFakeRepository({
      getMyPermissions: vi.fn().mockResolvedValue(ScyllaResult.success(permissions)),
    });
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useRefreshMyPermissions(), { wrapper: Wrapper });

    await result.current();

    expect(getMyPermissions).toHaveBeenCalled();
    expect(usePermissionsStore.getState().permissions).toEqual(permissions);
  });

  it('settles as "no permissions" (not stuck loading) when no user is signed in, without calling the backend', async () => {
    const { repository, getMyPermissions } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useRefreshMyPermissions(), { wrapper: Wrapper });

    await result.current();

    expect(getMyPermissions).not.toHaveBeenCalled();
    expect(usePermissionsStore.getState().permissions).toEqual({ scopes: [] });
  });

  it('settles as "no permissions" when the lookup itself fails, rather than leaving the old value stuck', async () => {
    localStorage.setItem('userId', 'user-1');
    const error = new ScyllaError('boom', { cause: { code: 'INTERNAL' } });
    const { repository } = makeFakeRepository({
      getMyPermissions: vi.fn().mockResolvedValue(ScyllaResult.error(error)),
    });
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useRefreshMyPermissions(), { wrapper: Wrapper });

    await result.current();

    expect(usePermissionsStore.getState().permissions).toEqual({ scopes: [] });
  });
});

describe('usePermissionVocabulary', () => {
  it('coherentAtScope is permissive (true) before the vocabulary has loaded', () => {
    const { repository } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => usePermissionVocabulary(), { wrapper: Wrapper });

    expect(result.current.coherentAtScope(0, PermissionScope.PROJECT)).toBe(true);
  });

  it('a permission is coherent at its own minScope and any broader (ancestor) scope, never narrower', async () => {
    const { repository } = makeFakeRepository({
      listPermissionVocabulary: vi.fn().mockResolvedValue(
        ScyllaResult.success({
          actions: [{ permission: 999 as never, minScope: PermissionScope.ORGANIZATION }],
        }),
      ),
    });
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => usePermissionVocabulary(), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(result.current.coherentAtScope(999 as never, PermissionScope.SYSTEM)).toBe(true);
    expect(result.current.coherentAtScope(999 as never, PermissionScope.ORGANIZATION)).toBe(true);
    expect(result.current.coherentAtScope(999 as never, PermissionScope.PROJECT)).toBe(false);
  });
});

describe('useEffectivePermissions', () => {
  it('is idle until called, then looks up the given principal', async () => {
    const { repository, getEffectivePermissions } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useEffectivePermissions(), { wrapper: Wrapper });

    expect(getEffectivePermissions).not.toHaveBeenCalled();

    await result.current.mutateAsync({ kind: PrincipalKind.USER, id: 'user-1' });
    expect(getEffectivePermissions).toHaveBeenCalledWith({ kind: PrincipalKind.USER, id: 'user-1' });
  });
});
