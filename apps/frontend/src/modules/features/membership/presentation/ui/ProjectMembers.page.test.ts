import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { Permission, PermissionScope, PrincipalKind, permissionsStore } from '@platform/authz';
import { contextStore } from '@platform/context';
import { render, withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import ProjectMembersPage from './ProjectMembers.page.svelte';

const projectGrant = (overrides: Record<string, unknown> = {}) => ({
  id: 'grant-1',
  principal: { kind: PrincipalKind.USER, id: 'user-1' },
  roleId: 'project-admin',
  scope: PermissionScope.PROJECT,
  scopeId: 'project-1',
  ...overrides,
});

const organizationGrant = (overrides: Record<string, unknown> = {}) => ({
  id: 'grant-org-1',
  principal: { kind: PrincipalKind.USER, id: 'user-2' },
  roleId: 'organization-admin',
  scope: PermissionScope.ORGANIZATION,
  scopeId: 'org-1',
  ...overrides,
});

let listProjectMembers: ReturnType<typeof vi.fn>;
let listOrganizationMembers: ReturnType<typeof vi.fn>;
let listGrants: ReturnType<typeof vi.fn>;
let listRoles: ReturnType<typeof vi.fn>;
let revokeAllAccess: ReturnType<typeof vi.fn>;
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
  listProjectMembers = vi
    .fn()
    .mockResolvedValue(ScyllaResult.success([{ userId: 'user-1', username: 'alice' }]));
  listOrganizationMembers = vi.fn().mockResolvedValue(
    ScyllaResult.success([
      { userId: 'user-1', username: 'alice' },
      { userId: 'user-2', username: 'bob' },
      { userId: 'user-3', username: 'carol' },
    ]),
  );

  // Scope decides which list answers: the project's own grants, or the
  // organization's (which only an org admin may read).
  listGrants = vi.fn().mockImplementation((scope?: PermissionScope) =>
    Promise.resolve(
      ScyllaResult.success(
        scope === PermissionScope.ORGANIZATION ? [organizationGrant()] : [projectGrant()],
      ),
    ),
  );
  listRoles = vi.fn().mockResolvedValue(
    ScyllaResult.success([
      {
        id: 'organization-admin',
        name: 'Organization admin',
        description: '',
        scope: PermissionScope.ORGANIZATION,
        origin: { kind: 'builtin' },
        access: { kind: 'fullControl' },
      },
    ]),
  );
  revokeAllAccess = vi.fn().mockResolvedValue(ScyllaResult.success(1));

  cache = withQueryClient();
  restoreRegistry = withRegistry({
    project: { projectRepository: { listMembers: listProjectMembers } },
    organization: { organizationRepository: { listMembers: listOrganizationMembers } },
    roles: {
      permissionRepository: {
        listGrants,
        listRoles,
        listGrantableRoles: vi.fn().mockResolvedValue(ScyllaResult.success([])),
        createGrant: vi.fn().mockResolvedValue(ScyllaResult.success(undefined)),
        revokeGrant: vi.fn().mockResolvedValue(ScyllaResult.success(undefined)),
        revokeAllAccess,
        getMyPermissions: vi.fn().mockResolvedValue(ScyllaResult.success({ scopes: [] })),
      },
    },
  });

  contextStore.setState({
    organization: { id: 'org-1', name: 'Acme' },
    project: { id: 'project-1', name: 'Web' },
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

describe('ProjectMembersPage', () => {
  it('renders nothing without a project id', () => {
    render(ProjectMembersPage, {});

    expect(listProjectMembers).not.toHaveBeenCalled();
    expect(screen.queryByRole('button', { name: 'Add a member' })).not.toBeInTheDocument();
  });

  it('lists the holders of a project-scoped grant', async () => {
    render(ProjectMembersPage, { projectId: 'project-1' });

    expect(await screen.findByText('alice')).toBeInTheDocument();
  });

  it('also lists someone who only reaches the project through an organization role', async () => {
    // ListProjectMembers alone would make the project look emptier — and more
    // locked down — than it is.
    render(ProjectMembersPage, { projectId: 'project-1' });

    expect(await screen.findByText('bob')).toBeInTheDocument();
  });

  it('leaves out an organization member whose role confers nothing here', async () => {
    render(ProjectMembersPage, { projectId: 'project-1' });

    await screen.findByText('alice');
    // carol holds no grant at all — listing her would read as access she does
    // not have.
    expect(screen.queryByText('carol')).not.toBeInTheDocument();
  });

  it('explains that inherited roles are visible and managed elsewhere', async () => {
    render(ProjectMembersPage, { projectId: 'project-1' });

    expect(
      await screen.findByText(/inherited from the organization and are managed there/),
    ).toBeInTheDocument();
  });

  it('says what it cannot show when the organization grants are out of reach', async () => {
    grant([
      Permission.LIST_PROJECT_MEMBERS,
      Permission.MANAGE_PROJECT_GRANTS,
      Permission.LIST_ORGANIZATION_MEMBERS,
    ]);
    render(ProjectMembersPage, { projectId: 'project-1' });

    expect(
      await screen.findByText(/Their roles are managed at the organization level/),
    ).toBeInTheDocument();
  });

  it('says so when nobody has access to the project yet', async () => {
    listProjectMembers.mockResolvedValue(ScyllaResult.success([]));
    listGrants.mockResolvedValue(ScyllaResult.success([]));
    render(ProjectMembersPage, { projectId: 'project-1' });

    expect(await screen.findByText('Nobody has access to this project yet.')).toBeInTheDocument();
  });

  it('disables adding a member without MANAGE_PROJECT_GRANTS', async () => {
    grant([Permission.LIST_PROJECT_MEMBERS, Permission.LIST_ORGANIZATION_MEMBERS]);
    render(ProjectMembersPage, { projectId: 'project-1' });

    await vi.waitFor(() =>
      expect(screen.getByRole('button', { name: 'Add a member' })).toBeDisabled(),
    );
  });

  it('removes only someone who holds a project-scoped grant', async () => {
    render(ProjectMembersPage, { projectId: 'project-1' });
    await screen.findByText('bob');

    // alice holds a direct grant; bob's role is inherited, so there is nothing
    // to remove from *this* project — exactly one control exists.
    const removals = screen.getAllByRole('button', { name: 'Remove from the project' });
    expect(removals).toHaveLength(1);

    await userEvent.click(removals[0]);
    await userEvent.click(await screen.findByRole('button', { name: 'Continue' }));

    await vi.waitFor(() =>
      expect(revokeAllAccess).toHaveBeenCalledWith({
        principal: { kind: PrincipalKind.USER, id: 'user-1' },
        scope: PermissionScope.PROJECT,
        scopeId: 'project-1',
      }),
    );
  });
});
