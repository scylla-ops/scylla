import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { flushSync } from 'svelte';
import { PermissionScope, permissionsStore } from '@platform/authz';
import { withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { RoleEntity } from '@/modules/features/roles';
import { createAssignableRoles } from '../assignable-roles.state.svelte.ts';

interface GrantableRole {
  roleId: string;
  scope: PermissionScope;
  kind: number;
  description: string;
}

const role = (overrides: Partial<RoleEntity> = {}): RoleEntity =>
  ({
    id: 'role-1',
    name: 'Custom name',
    description: 'Custom description',
    scope: PermissionScope.PROJECT,
    origin: { kind: 'custom' },
    access: { kind: 'fullControl' },
    ...overrides,
  });

const grantable = (overrides: Partial<GrantableRole> = {}): GrantableRole => ({
  roleId: 'project-viewer',
  scope: PermissionScope.PROJECT,
  kind: 0,
  description: 'Read-only access',
  ...overrides,
});

let listRoles: ReturnType<typeof vi.fn>;
let listGrantableRoles: ReturnType<typeof vi.fn>;
let cache: ReturnType<typeof withQueryClient>;
let restoreRegistry: () => void;

const grantCatalogAccess = () =>
  permissionsStore.setState({
    permissions: {
      scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }],
    },
  });

const denyCatalogAccess = () => permissionsStore.setState({ permissions: { scopes: [] } });

beforeEach(() => {
  listRoles = vi.fn().mockResolvedValue(ScyllaResult.success([]));
  listGrantableRoles = vi.fn().mockResolvedValue(ScyllaResult.success([]));

  cache = withQueryClient();
  restoreRegistry = withRegistry({
    roles: { permissionRepository: { listRoles, listGrantableRoles } },
  });

  denyCatalogAccess();
});

afterEach(() => {
  cache.restore();
  restoreRegistry();
  permissionsStore.setState({ permissions: null });
});

/** Runs `body` in a reactive root and waits for both queries. */
const withAssignableRoles = async (
  scope: PermissionScope,
  body: (state: ReturnType<typeof createAssignableRoles>) => void | Promise<void>,
) => {
  let state!: ReturnType<typeof createAssignableRoles>;
  const cleanup = $effect.root(() => {
    state = createAssignableRoles(scope);
  });

  try {
    await vi.waitFor(() => {
      flushSync();
      expect(state.isLoading).toBe(false);
    });
    await body(state);
  } finally {
    cleanup();
  }
};

describe('createAssignableRoles', () => {
  it('never asks for the catalog without MANAGE_ROLES — a denial is not worth requesting', async () => {
    await withAssignableRoles(PermissionScope.PROJECT, () => {
      expect(listRoles).not.toHaveBeenCalled();
      expect(listGrantableRoles).toHaveBeenCalled();
    });
  });

  it('asks for the catalog once the caller holds MANAGE_ROLES', async () => {
    grantCatalogAccess();

    await withAssignableRoles(PermissionScope.PROJECT, async () => {
      await vi.waitFor(() => expect(listRoles).toHaveBeenCalled());
    });
  });

  it('lists the builtin grantable roles with no catalog access, humanizing an unmatched id', async () => {
    listGrantableRoles.mockResolvedValue(
      ScyllaResult.success([grantable({ roleId: 'project-viewer' })]),
    );

    await withAssignableRoles(PermissionScope.PROJECT, state => {
      expect(state.assignableRoles).toEqual([
        {
          roleId: 'project-viewer',
          name: 'Project viewer',
          description: 'Read-only access',
          role: undefined,
        },
      ]);
    });
  });

  it("prefers the catalog's own name and description when both lists have the role", async () => {
    grantCatalogAccess();
    const catalogRole = role({
      id: 'role-1',
      name: 'Catalog name',
      description: 'Catalog description',
    });
    listGrantableRoles.mockResolvedValue(
      ScyllaResult.success([grantable({ roleId: 'role-1', description: 'grantable-only' })]),
    );
    listRoles.mockResolvedValue(ScyllaResult.success([catalogRole]));

    await withAssignableRoles(PermissionScope.PROJECT, async state => {
      await vi.waitFor(() => {
        flushSync();
        expect(state.assignableRoles[0]).toEqual({
          roleId: 'role-1',
          name: 'Catalog name',
          description: 'Catalog description',
          role: catalogRole,
        });
      });
    });
  });

  it('adds a custom role bound to this scope that the grantable list never carries', async () => {
    grantCatalogAccess();
    listRoles.mockResolvedValue(
      ScyllaResult.success([role({ id: 'custom-1', scope: PermissionScope.PROJECT })]),
    );

    await withAssignableRoles(PermissionScope.PROJECT, async state => {
      await vi.waitFor(() => {
        flushSync();
        expect(state.assignableRoles.map(entry => entry.roleId)).toEqual(['custom-1']);
      });
    });
  });

  it('excludes a custom role bound to a different scope', async () => {
    grantCatalogAccess();
    listRoles.mockResolvedValue(
      ScyllaResult.success([role({ id: 'org-role', scope: PermissionScope.ORGANIZATION })]),
    );

    await withAssignableRoles(PermissionScope.PROJECT, async state => {
      await vi.waitFor(() => expect(listRoles).toHaveBeenCalled());
      flushSync();
      expect(state.assignableRoles).toEqual([]);
    });
  });

  it('labelFor resolves a role through the assignable list first', async () => {
    grantCatalogAccess();
    listGrantableRoles.mockResolvedValue(ScyllaResult.success([grantable({ roleId: 'role-1' })]));
    listRoles.mockResolvedValue(ScyllaResult.success([role({ id: 'role-1', name: 'From catalog' })]));

    await withAssignableRoles(PermissionScope.PROJECT, async state => {
      await vi.waitFor(() => {
        flushSync();
        expect(state.labelFor('role-1')).toBe('From catalog');
      });
    });
  });

  it('labelFor still names a role bound to another scope — known, just not assignable here', async () => {
    grantCatalogAccess();
    listRoles.mockResolvedValue(
      ScyllaResult.success([
        role({ id: 'org-role', scope: PermissionScope.ORGANIZATION, name: 'Org-only role' }),
      ]),
    );

    await withAssignableRoles(PermissionScope.PROJECT, async state => {
      await vi.waitFor(() => {
        flushSync();
        expect(state.labelFor('org-role')).toBe('Org-only role');
      });
    });
  });

  it('labelFor falls back to a humanized id when the role is unknown everywhere', async () => {
    await withAssignableRoles(PermissionScope.PROJECT, state => {
      expect(state.labelFor('mystery-role')).toBe('Mystery role');
    });
  });
});
