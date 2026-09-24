import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { waitFor } from '@testing-library/svelte';
import {
  Permission,
  PermissionScope,
  PrincipalKind,
  permissionsStore,
} from '@platform/authz';
import { selectionStore } from '@shared/presentation/stores/selection.store.ts';
import { withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PermissionRepository } from '../domain/repository/permission.repository.ts';
import type { RoleEntity } from '../domain/entities/role.entity.ts';
import type { GrantEntity } from '../domain/entities/grant.entity.ts';
import { createRolesPage } from './roles-page.state.svelte.ts';

const role = (overrides: Partial<RoleEntity> = {}): RoleEntity => ({
  id: 'role-1',
  name: 'viewer',
  description: 'read only',
  scope: PermissionScope.ORGANIZATION,
  origin: { kind: 'custom' },
  access: { kind: 'restricted', permissions: [Permission.READ_ORGANIZATION] },
  ...overrides,
});

const grant = (overrides: Partial<GrantEntity> = {}): GrantEntity => ({
  id: 'grant-1',
  principal: { kind: PrincipalKind.USER, id: 'user-1' },
  roleId: 'role-1',
  scope: PermissionScope.ORGANIZATION,
  scopeId: 'org-1',
  ...overrides,
});

let listRoles: ReturnType<typeof vi.fn>;
let listGrants: ReturnType<typeof vi.fn>;
let deleteRole: ReturnType<typeof vi.fn>;
let cache: ReturnType<typeof withQueryClient>;
let restoreRegistry: () => void;

const asSystemAdmin = () =>
  permissionsStore.setState({
    permissions: {
      scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }],
    },
  });

beforeEach(() => {
  listRoles = vi
    .fn()
    .mockResolvedValue(
      ScyllaResult.success([role(), role({ id: 'role-2', name: 'admin', origin: { kind: 'builtin', key: 'organization-admin' } })]),
    );
  listGrants = vi.fn().mockResolvedValue(ScyllaResult.success([grant()]));
  deleteRole = vi.fn().mockResolvedValue(ScyllaResult.success(undefined));

  cache = withQueryClient();
  restoreRegistry = withRegistry({
    roles: {
      permissionRepository: {
        listRoles,
        listGrants,
        deleteRole,
      } as unknown as PermissionRepository,
      updateRole: { execute: vi.fn() },
    },
  });

  selectionStore.setState({ selectedIds: {} });
  asSystemAdmin();
});

afterEach(() => {
  cache.restore();
  restoreRegistry();
  permissionsStore.setState({ permissions: null });
});

/** Builds the ViewModel inside a reactive root — it is made of runes. */
const withPage = async (
  body: (page: ReturnType<typeof createRolesPage>) => Promise<void> | void,
) => {
  let page!: ReturnType<typeof createRolesPage>;
  const cleanup = $effect.root(() => {
    page = createRolesPage();
  });

  try {
    await body(page);
  } finally {
    cleanup();
  }
};

describe('createRolesPage', () => {
  it('fetches the catalog and every grant once, for both halves of the screen', async () => {
    await withPage(async page => {
      await waitFor(() => expect(page.roles).toHaveLength(2));
      expect(listRoles).toHaveBeenCalledTimes(1);
      expect(listGrants).toHaveBeenCalledTimes(1);
    });
  });

  it('counts holders from the grant list it already needs, with no extra request', async () => {
    listGrants.mockResolvedValue(
      ScyllaResult.success([
        grant(),
        grant({ id: 'grant-2', principal: { kind: PrincipalKind.USER, id: 'user-2' } }),
        grant({ id: 'grant-3', roleId: 'role-2' }),
      ]),
    );

    await withPage(async page => {
      await waitFor(() => expect(page.memberCountOf('role-1')).toBe(2));
      expect(page.memberCountOf('role-2')).toBe(1);
      expect(page.memberCountOf('role-404')).toBe(0);
    });
  });

  it('never offers a builtin role for deletion — the backend compiles it in', async () => {
    await withPage(async page => {
      await waitFor(() => expect(page.roles).toHaveLength(2));

      expect(page.isSelectable(page.roles[0])).toBe(true);
      expect(page.isSelectable(page.roles[1])).toBe(false);
    });
  });

  it('offers nothing for deletion without MANAGE_ROLES, builtin or not', async () => {
    permissionsStore.setState({
      permissions: {
        scopes: [
          {
            scope: PermissionScope.SYSTEM,
            scopeId: '',
            access: { kind: 'restricted', permissions: [Permission.LIST_USERS] },
          },
        ],
      },
    });

    await withPage(async page => {
      await waitFor(() => expect(page.roles).toHaveLength(2));

      expect(page.canManageRoles).toBe(false);
      expect(page.isSelectable(page.roles[0])).toBe(false);
    });
  });

  it('resolves the open role from the catalog, so an edit shows the fresh copy', async () => {
    await withPage(async page => {
      await waitFor(() => expect(page.roles).toHaveLength(2));

      expect(page.activeRole).toBeNull();
      page.open('role-2');
      expect(page.activeRole?.name).toBe('admin');
    });
  });

  it('opens the form on nothing when creating, and on the role when editing', async () => {
    await withPage(async page => {
      await waitFor(() => expect(page.roles).toHaveLength(2));

      page.openCreate();
      expect(page.formOpen).toBe(true);
      expect(page.editingRole).toBeNull();

      page.openEdit(page.roles[0]);
      expect(page.editingRole?.id).toBe('role-1');

      page.closeForm();
      expect(page.formOpen).toBe(false);
    });
  });

  it('deletes the selected roles through the repository', async () => {
    await withPage(async page => {
      await waitFor(() => expect(page.roles).toHaveLength(2));

      page.selection.select('role-1');
      await page.selection.headerProps.onDeleteSelection?.();

      expect(deleteRole).toHaveBeenCalledWith('role-1');
    });
  });
});
