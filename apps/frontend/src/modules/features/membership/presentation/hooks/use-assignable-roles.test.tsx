import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook } from '@testing-library/react';
import { PermissionScope, usePermissionsStore } from '@platform/authz';
import { useAssignableRoles } from './use-assignable-roles';
import type * as RolesModule from '@/modules/features/roles';
import type { RoleEntity } from '@/modules/features/roles';

// `GrantableRoleEntity` isn't part of the roles feature's public API (its
// barrel only re-exports the hook, never the entity) — the real
// useAssignableRoles.ts never names the type either, it just flows through
// useGrantableRoles's return shape. Derive it the same way rather than
// reaching past the barrel.
type GrantableRoleEntity = ReturnType<typeof RolesModule.useGrantableRoles>['grantableRoles'][number];

let rolesFixture: RoleEntity[] = [];
let grantableFixture: GrantableRoleEntity[] = [];
const useRolesMock = vi.fn((_options?: { enabled?: boolean }) => ({ roles: rolesFixture }));

vi.mock('@/modules/features/roles', async importOriginal => {
  const actual = await importOriginal<typeof RolesModule>();
  return {
    ...actual,
    useRoles: (options?: { enabled?: boolean }) => useRolesMock(options),
    useGrantableRoles: () => ({ grantableRoles: grantableFixture, isLoading: false }),
  };
});

const role = (overrides: Partial<RoleEntity> = {}): RoleEntity => ({
  id: 'role-1',
  name: 'Custom name',
  description: 'Custom description',
  scope: PermissionScope.PROJECT,
  origin: { kind: 'custom' },
  access: { kind: 'fullControl' },
  ...overrides,
});

const grantable = (overrides: Partial<GrantableRoleEntity> = {}): GrantableRoleEntity => ({
  roleId: 'project-viewer',
  scope: PermissionScope.PROJECT,
  kind: 0,
  description: 'Read-only access',
  ...overrides,
});

beforeEach(() => {
  useRolesMock.mockClear();
  rolesFixture = [];
  grantableFixture = [];
});

const grantFullCatalogAccess = () => {
  usePermissionsStore.setState({
    permissions: {
      scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }],
    },
  });
};

const denyFullCatalogAccess = () => {
  usePermissionsStore.setState({ permissions: { scopes: [] } });
};

describe('useAssignableRoles', () => {
  it('asks useRoles for enabled:false when the caller lacks MANAGE_ROLES (never asks for a denial)', () => {
    denyFullCatalogAccess();
    renderHook(() => useAssignableRoles(PermissionScope.PROJECT));
    expect(useRolesMock).toHaveBeenCalledWith({ enabled: false });
  });

  it('asks useRoles for enabled:true when the caller holds MANAGE_ROLES', () => {
    grantFullCatalogAccess();
    renderHook(() => useAssignableRoles(PermissionScope.PROJECT));
    expect(useRolesMock).toHaveBeenCalledWith({ enabled: true });
  });

  it('lists the builtin grantable roles even with no catalog access, humanizing an id with no catalog match', () => {
    denyFullCatalogAccess();
    grantableFixture = [grantable({ roleId: 'project-viewer' })];

    const { result } = renderHook(() => useAssignableRoles(PermissionScope.PROJECT));

    expect(result.current.assignableRoles).toEqual([
      {
        roleId: 'project-viewer',
        name: 'Project viewer',
        description: 'Read-only access',
        role: undefined,
      },
    ]);
  });

  it('enriches a grantable entry with the catalog\'s own name/description when both are available', () => {
    grantFullCatalogAccess();
    grantableFixture = [grantable({ roleId: 'role-1', description: 'grantable-only description' })];
    rolesFixture = [role({ id: 'role-1', name: 'Catalog name', description: 'Catalog description' })];

    const { result } = renderHook(() => useAssignableRoles(PermissionScope.PROJECT));

    expect(result.current.assignableRoles[0]).toEqual({
      roleId: 'role-1',
      name: 'Catalog name',
      description: 'Catalog description',
      role: rolesFixture[0],
    });
  });

  it('adds a custom role bound to this scope that is absent from the grantable list', () => {
    grantFullCatalogAccess();
    grantableFixture = [];
    rolesFixture = [role({ id: 'custom-1', scope: PermissionScope.PROJECT })];

    const { result } = renderHook(() => useAssignableRoles(PermissionScope.PROJECT));

    expect(result.current.assignableRoles.map(r => r.roleId)).toEqual(['custom-1']);
  });

  it('excludes a custom role bound to a DIFFERENT scope', () => {
    grantFullCatalogAccess();
    rolesFixture = [role({ id: 'org-role', scope: PermissionScope.ORGANIZATION })];

    const { result } = renderHook(() => useAssignableRoles(PermissionScope.PROJECT));

    expect(result.current.assignableRoles).toEqual([]);
  });

  it('labelFor resolves a role via the assignable list first', () => {
    grantFullCatalogAccess();
    grantableFixture = [grantable({ roleId: 'role-1' })];
    rolesFixture = [role({ id: 'role-1', name: 'From catalog' })];

    const { result } = renderHook(() => useAssignableRoles(PermissionScope.PROJECT));
    expect(result.current.labelFor('role-1')).toBe('From catalog');
  });

  it('labelFor falls back to the raw catalog name for a role bound to another scope (not assignable here, but still known)', () => {
    grantFullCatalogAccess();
    rolesFixture = [role({ id: 'org-role', scope: PermissionScope.ORGANIZATION, name: 'Org-only role' })];

    const { result } = renderHook(() => useAssignableRoles(PermissionScope.PROJECT));
    expect(result.current.labelFor('org-role')).toBe('Org-only role');
  });

  it('labelFor falls back to a humanized id when the role is unknown everywhere', () => {
    const { result } = renderHook(() => useAssignableRoles(PermissionScope.PROJECT));
    expect(result.current.labelFor('mystery-role')).toBe('Mystery role');
  });
});
