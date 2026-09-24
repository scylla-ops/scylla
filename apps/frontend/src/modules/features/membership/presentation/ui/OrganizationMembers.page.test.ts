import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { Permission, PermissionScope, PrincipalKind, permissionsStore } from '@platform/authz';
import { contextStore } from '@platform/context';
import { findFloating, render, withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import OrganizationMembersPage from './OrganizationMembers.page.svelte';

const member = (overrides: Record<string, unknown> = {}) => ({
  userId: 'user-1',
  username: 'alice',
  ...overrides,
});

const grantEntity = (overrides: Record<string, unknown> = {}) => ({
  id: 'grant-1',
  principal: { kind: PrincipalKind.USER, id: 'user-1' },
  roleId: 'organization-admin',
  scope: PermissionScope.ORGANIZATION,
  scopeId: 'org-1',
  ...overrides,
});

let listMembers: ReturnType<typeof vi.fn>;
let getAll: ReturnType<typeof vi.fn>;
let listGrants: ReturnType<typeof vi.fn>;
let listGrantableRoles: ReturnType<typeof vi.fn>;
let revokeAllAccess: ReturnType<typeof vi.fn>;
let createGrant: ReturnType<typeof vi.fn>;
let cache: ReturnType<typeof withQueryClient>;
let restoreRegistry: () => void;

const grant = (permissions: Permission[] | 'all') =>
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
  listMembers = vi.fn().mockResolvedValue(ScyllaResult.success([member()]));
  getAll = vi
    .fn()
    .mockResolvedValue(
      ScyllaResult.success([
        { id: 'user-1', username: 'alice' },
        { id: 'user-2', username: 'bob' },
      ]),
    );
  listGrants = vi.fn().mockResolvedValue(ScyllaResult.success([grantEntity()]));
  listGrantableRoles = vi.fn().mockResolvedValue(
    ScyllaResult.success([
      {
        roleId: 'organization-member',
        scope: PermissionScope.ORGANIZATION,
        kind: 0,
        description: 'Belongs here',
      },
    ]),
  );
  revokeAllAccess = vi.fn().mockResolvedValue(ScyllaResult.success(2));
  createGrant = vi.fn().mockResolvedValue(ScyllaResult.success(undefined));

  cache = withQueryClient();
  restoreRegistry = withRegistry({
    organization: { organizationRepository: { listMembers } },
    user: { userRepository: { getAll } },
    roles: {
      permissionRepository: {
        listGrants,
        listGrantableRoles,
        listRoles: vi.fn().mockResolvedValue(ScyllaResult.success([])),
        createGrant,
        revokeGrant: vi.fn(),
        revokeAllAccess,
        getMyPermissions: vi.fn().mockResolvedValue(ScyllaResult.success({ scopes: [] })),
      },
    },
  });

  contextStore.setState({
    organization: { id: 'org-1', name: 'Acme' },
    project: { id: null, name: null },
  });
  localStorage.setItem('userId', 'user-9');
  grant('all');
});

afterEach(() => {
  cache.restore();
  restoreRegistry();
  localStorage.clear();
  permissionsStore.setState({ permissions: null });
});

describe('OrganizationMembersPage', () => {
  it('asks for an organization rather than rendering an empty list', () => {
    contextStore.setState({
      organization: { id: null, name: null },
      project: { id: null, name: null },
    });
    render(OrganizationMembersPage);

    expect(screen.getByText('Select an organization to see its members.')).toBeInTheDocument();
    expect(listMembers).not.toHaveBeenCalled();
  });

  it("lists the organization's members", async () => {
    render(OrganizationMembersPage);

    expect(await screen.findByText('alice')).toBeInTheDocument();
    expect(listMembers).toHaveBeenCalledWith('org-1');
  });

  it('says so when nobody belongs to the organization yet', async () => {
    listMembers.mockResolvedValue(ScyllaResult.success([]));
    listGrants.mockResolvedValue(ScyllaResult.success([]));
    render(OrganizationMembersPage);

    expect(await screen.findByText('Nobody belongs to this organization yet.')).toBeInTheDocument();
  });

  it('does not enumerate accounts without LIST_USERS', async () => {
    grant([Permission.MANAGE_ORG_GRANTS]);
    render(OrganizationMembersPage);

    await screen.findByText('alice');
    // Roles can still be moved among people already here — the directory just
    // stays out of reach.
    expect(getAll).not.toHaveBeenCalled();
  });

  it('offers to add a member when the caller may manage grants', async () => {
    render(OrganizationMembersPage);
    await screen.findByText('alice');

    expect(screen.getByRole('button', { name: 'Add a member' })).toBeEnabled();
  });

  it('disables adding a member without MANAGE_ORG_GRANTS', async () => {
    grant([Permission.LIST_USERS]);
    render(OrganizationMembersPage);

    await vi.waitFor(() => expect(screen.getByRole('button', { name: 'Add a member' })).toBeDisabled());
  });

  it('confirms before removing a member, and revokes all their access on continue', async () => {
    render(OrganizationMembersPage);
    await screen.findByText('alice');

    await userEvent.click(screen.getByRole('button', { name: 'Remove from the organization' }));
    expect(revokeAllAccess).not.toHaveBeenCalled();

    await userEvent.click(await screen.findByRole('button', { name: 'Continue' }));

    // RevokeAllAccess, not a series of revokeGrant: a narrower grant left
    // behind would keep them listed.
    await vi.waitFor(() =>
      expect(revokeAllAccess).toHaveBeenCalledWith({
        principal: { kind: PrincipalKind.USER, id: 'user-1' },
        scope: PermissionScope.ORGANIZATION,
        scopeId: 'org-1',
      }),
    );
  });

  it('grants a role to somebody already listed, without opening the dialog', async () => {
    render(OrganizationMembersPage);
    await screen.findByText('alice');

    await userEvent.click(screen.getByRole('combobox'));
    await userEvent.click(await findFloating('option', 'Organization member'));

    await vi.waitFor(() =>
      expect(createGrant).toHaveBeenCalledWith(
        expect.objectContaining({ roleId: 'organization-member', scopeId: 'org-1' }),
      ),
    );
  });
});
