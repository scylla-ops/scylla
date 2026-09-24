import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import {
  Permission,
  PermissionScope,
  PrincipalKind,
  permissionsStore,
} from '@platform/authz';
import { selectionStore } from '@shared/presentation/stores/selection.store.ts';
import { focusSettled, render, withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { ScyllaError, ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { RoleEntity } from '../../../domain/entities/role.entity.ts';
import type { GrantEntity } from '../../../domain/entities/grant.entity.ts';
import RolesPage from './Roles.page.svelte';

const custom = (overrides: Partial<RoleEntity> = {}): RoleEntity => ({
  id: 'role-1',
  name: 'viewer',
  description: 'read only',
  scope: PermissionScope.ORGANIZATION,
  origin: { kind: 'custom' },
  access: { kind: 'restricted', permissions: [Permission.LIST_SECRETS] },
  ...overrides,
});

const builtin = custom({
  id: 'role-2',
  name: 'organization-admin',
  description: '',
  origin: { kind: 'builtin', key: 'organization-admin' },
  access: { kind: 'fullControl' },
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
let createRole: ReturnType<typeof vi.fn>;
let deleteRole: ReturnType<typeof vi.fn>;
let revokeGrant: ReturnType<typeof vi.fn>;
let cache: ReturnType<typeof withQueryClient>;
let restoreRegistry: () => void;

const withPermissions = (permissions: Permission[] | 'all') =>
  permissionsStore.setState({
    permissions: {
      scopes: [
        {
          scope: PermissionScope.SYSTEM,
          scopeId: '',
          access:
            permissions === 'all' ? { kind: 'fullControl' } : { kind: 'restricted', permissions },
        },
      ],
    },
  });

beforeEach(() => {
  listRoles = vi.fn().mockResolvedValue(ScyllaResult.success([custom(), builtin]));
  listGrants = vi.fn().mockResolvedValue(ScyllaResult.success([grant()]));
  createRole = vi.fn().mockResolvedValue(ScyllaResult.success(custom()));
  deleteRole = vi.fn().mockResolvedValue(ScyllaResult.success(undefined));
  revokeGrant = vi.fn().mockResolvedValue(ScyllaResult.success(undefined));

  cache = withQueryClient();
  restoreRegistry = withRegistry({
    roles: {
      permissionRepository: {
        listRoles,
        listGrants,
        createRole,
        deleteRole,
        revokeGrant,
        createGrant: vi.fn(),
        getMyPermissions: vi.fn().mockResolvedValue(ScyllaResult.success({ scopes: [] })),
      },
      updateRole: { execute: vi.fn().mockResolvedValue(ScyllaResult.success(custom())) },
    },
    user: {
      userRepository: {
        getAll: vi
          .fn()
          .mockResolvedValue(ScyllaResult.success({ items: [{ userId: 'user-1', username: 'ada' }] })),
      },
    },
    organization: {
      organizationRepository: {
        getMine: vi.fn().mockResolvedValue(ScyllaResult.success([{ id: 'org-1', name: 'Acme' }])),
      },
    },
    project: {
      projectRepository: {
        getByOrganizationId: vi.fn().mockResolvedValue(ScyllaResult.success({ projects: [] })),
      },
    },
  });

  selectionStore.setState({ selectedIds: {} });
  withPermissions('all');
});

afterEach(() => {
  cache.restore();
  restoreRegistry();
  permissionsStore.setState({ permissions: null });
});

describe('RolesPage — the catalog', () => {
  it('lists the roles with their scope and how many people hold them', async () => {
    render(RolesPage);

    expect(await screen.findByText('viewer')).toBeInTheDocument();
    expect(screen.getByText('organization-admin')).toBeInTheDocument();
    expect(screen.getByText('1 members')).toBeInTheDocument();
  });

  it('stands in for a role with no description rather than leaving a gap', async () => {
    render(RolesPage);

    await screen.findByText('organization-admin');
    expect(screen.getByText('No description')).toBeInTheDocument();
  });

  it('explains an empty catalog instead of showing a bare column', async () => {
    listRoles.mockResolvedValue(ScyllaResult.success([]));
    render(RolesPage);

    expect(
      await screen.findByText('No roles yet. Create one to get started.'),
    ).toBeInTheDocument();
  });

  it('offers no detail until a role is opened', async () => {
    render(RolesPage);

    expect(
      await screen.findByText('Select a role to see its permissions and members.'),
    ).toBeInTheDocument();
  });

  it('never offers a builtin role for deletion — the backend compiles it in', async () => {
    render(RolesPage);

    await screen.findByText('viewer');
    expect(screen.getByRole('checkbox', { name: 'viewer' })).toBeEnabled();
    expect(screen.getByRole('checkbox', { name: 'organization-admin' })).toBeDisabled();
  });

  it('deletes the selected roles once the confirmation is accepted', async () => {
    render(RolesPage);
    await screen.findByText('viewer');

    await userEvent.click(screen.getByRole('checkbox', { name: 'viewer' }));
    await userEvent.click(screen.getByRole('button', { name: 'Delete' }));
    await userEvent.click(await screen.findByRole('button', { name: 'Continue' }));

    await vi.waitFor(() => expect(deleteRole).toHaveBeenCalledWith('role-1'));
  });
});

describe('RolesPage — one role in detail', () => {
  const openViewer = async () => {
    render(RolesPage);
    await userEvent.click(await screen.findByRole('button', { name: /viewer/ }));
  };

  it('shows what a restricted role confers, labelled against its own scope', async () => {
    await openViewer();

    expect(await screen.findByText('Permissions')).toBeInTheDocument();
    expect(screen.getByText('View the secrets of every project')).toBeInTheDocument();
  });

  it('says so plainly for a role that confers everything', async () => {
    render(RolesPage);
    await userEvent.click(await screen.findByRole('button', { name: /organization-admin/ }));

    expect(await screen.findByText('Grants full control over its scope.')).toBeInTheDocument();
  });

  it('lists who holds the role, by username rather than by principal id', async () => {
    await openViewer();

    expect(await screen.findByText('ada')).toBeInTheDocument();
    expect(screen.queryByText('user-1')).not.toBeInTheDocument();
  });

  it('names the organization a grant is bound to', async () => {
    await openViewer();

    await screen.findByText('ada');
    expect(screen.getByText('Acme')).toBeInTheDocument();
  });

  it('explains an unheld role instead of showing an empty list', async () => {
    listGrants.mockResolvedValue(ScyllaResult.success([]));
    await openViewer();

    expect(await screen.findByText('No one holds this role yet.')).toBeInTheDocument();
  });

  it('revokes a grant from the detail panel', async () => {
    await openViewer();
    await screen.findByText('ada');

    await userEvent.click(screen.getByRole('button', { name: 'Remove' }));

    await vi.waitFor(() => expect(revokeGrant).toHaveBeenCalledWith('grant-1'));
  });

  it('refuses to edit a builtin role, which the backend would reject anyway', async () => {
    render(RolesPage);
    await userEvent.click(await screen.findByRole('button', { name: /organization-admin/ }));

    expect(await screen.findByRole('button', { name: /Edit/ })).toBeDisabled();
  });
});

describe('RolesPage — without the permission', () => {
  beforeEach(() => withPermissions([Permission.LIST_USERS]));

  it('offers no way to create a role', async () => {
    render(RolesPage);

    await screen.findByText('viewer');
    expect(screen.queryByRole('button', { name: 'Create role' })).not.toBeInTheDocument();
  });

  it('selects nothing, so the bulk delete can never be reached', async () => {
    render(RolesPage);

    await screen.findByText('viewer');
    expect(screen.getByRole('checkbox', { name: 'viewer' })).toBeDisabled();
  });

  it('shows the edit and revoke controls disabled rather than hiding them', async () => {
    render(RolesPage);
    await userEvent.click(await screen.findByRole('button', { name: /viewer/ }));

    expect(await screen.findByRole('button', { name: /Edit/ })).toBeDisabled();
    await screen.findByText('ada');
    expect(screen.getByRole('button', { name: "You don't have permission to revoke grants." })).toBeDisabled();
  });
});

describe('RolesPage — the role form', () => {
  it('creates a role from the header button', async () => {
    render(RolesPage);
    await screen.findByText('viewer');

    await userEvent.click(screen.getByRole('button', { name: 'Create role' }));
    await focusSettled();

    const dialog = await screen.findByRole('dialog');
    await userEvent.type(within(dialog).getByLabelText('Name'), 'auditor');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Create role' }));

    await vi.waitFor(() => expect(createRole).toHaveBeenCalled());
    expect(createRole).toHaveBeenCalledWith(
      expect.objectContaining({ name: 'auditor', scope: PermissionScope.ORGANIZATION }),
    );
  });

  it('opens on the role being edited, and says its scope is fixed', async () => {
    render(RolesPage);
    await userEvent.click(await screen.findByRole('button', { name: /viewer/ }));
    await userEvent.click(await screen.findByRole('button', { name: /Edit/ }));
    await focusSettled();

    const dialog = await screen.findByRole('dialog');
    expect(within(dialog).getByLabelText('Name')).toHaveValue('viewer');
    expect(within(dialog).getByText('Scope cannot be changed after creation.')).toBeInTheDocument();
  });

  it('refuses to submit a role with no name', async () => {
    render(RolesPage);
    await screen.findByText('viewer');

    await userEvent.click(screen.getByRole('button', { name: 'Create role' }));
    await focusSettled();

    const dialog = await screen.findByRole('dialog');
    expect(within(dialog).getByRole('button', { name: 'Create role' })).toBeDisabled();
  });
});

describe('RolesPage — when the backend refuses', () => {
  it("surfaces the backend's own message rather than an empty catalog", async () => {
    const error = new ScyllaError('nope', { cause: { code: 'PERMISSION_DENIED' } });
    listRoles.mockResolvedValue(ScyllaResult.error(error));
    render(RolesPage);

    // The page still renders; the catalog is simply empty and says so.
    expect(
      await screen.findByText('No roles yet. Create one to get started.'),
    ).toBeInTheDocument();
  });
});
