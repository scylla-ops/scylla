import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { waitFor } from '@testing-library/svelte';
import {
  Permission,
  PermissionScope,
  PrincipalKind,
  permissionsStore,
} from '@platform/authz';
import { withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { RoleEntity } from '../../domain/entities/role.entity.ts';
import type { GrantEntity } from '../../domain/entities/grant.entity.ts';
import { createGrantTargetLabels } from '../grant-target-labels.svelte.ts';
import { createRoleAssignees } from '../role-assignees.state.svelte.ts';

const role = (overrides: Partial<RoleEntity> = {}): RoleEntity => ({
  id: 'role-1',
  name: 'viewer',
  description: '',
  scope: PermissionScope.ORGANIZATION,
  origin: { kind: 'custom' },
  access: { kind: 'restricted', permissions: [Permission.LIST_SECRETS] },
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

let listGrants: ReturnType<typeof vi.fn>;
let revokeGrant: ReturnType<typeof vi.fn>;
let getAllUsers: ReturnType<typeof vi.fn>;
let getMine: ReturnType<typeof vi.fn>;
let getProjects: ReturnType<typeof vi.fn>;
let cache: ReturnType<typeof withQueryClient>;
let restoreRegistry: () => void;

beforeEach(() => {
  listGrants = vi.fn().mockResolvedValue(ScyllaResult.success([grant()]));
  revokeGrant = vi.fn().mockResolvedValue(ScyllaResult.success(undefined));
  getAllUsers = vi
    .fn()
    .mockResolvedValue(ScyllaResult.success({ items: [{ userId: 'user-1', username: 'ada' }] }));
  getMine = vi.fn().mockResolvedValue(ScyllaResult.success([{ id: 'org-1', name: 'Acme' }]));
  getProjects = vi
    .fn()
    .mockResolvedValue(
      ScyllaResult.success({ projects: [{ id: 'project-1', name: 'Web', organizationId: 'org-1' }] }),
    );

  cache = withQueryClient();
  restoreRegistry = withRegistry({
    roles: {
      permissionRepository: {
        listGrants,
        revokeGrant,
        listRoles: vi.fn().mockResolvedValue(ScyllaResult.success([])),
        getMyPermissions: vi.fn().mockResolvedValue(ScyllaResult.success({ scopes: [] })),
      },
      updateRole: { execute: vi.fn() },
    },
    user: { userRepository: { getAll: getAllUsers } },
    organization: { organizationRepository: { getMine } },
    project: { projectRepository: { getByOrganizationId: getProjects } },
  });

  permissionsStore.setState({
    permissions: {
      scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }],
    },
  });
});

afterEach(() => {
  cache.restore();
  restoreRegistry();
  permissionsStore.setState({ permissions: null });
});

const inRoot = async <T>(build: () => T, body: (value: T) => Promise<void> | void) => {
  let value!: T;
  const cleanup = $effect.root(() => {
    value = build();
  });

  try {
    await body(value);
  } finally {
    cleanup();
  }
};

describe('createRoleAssignees', () => {
  it('keeps only the grants of the role it was built for', async () => {
    listGrants.mockResolvedValue(
      ScyllaResult.success([grant(), grant({ id: 'grant-2', roleId: 'role-other' })]),
    );

    await inRoot(
      () => createRoleAssignees(() => role()),
      async assignees => {
        await waitFor(() => expect(assignees.assignees).toHaveLength(1));
        expect(assignees.assignees[0].grant.id).toBe('grant-1');
      },
    );
  });

  it('resolves a user principal to their username', async () => {
    await inRoot(
      () => createRoleAssignees(() => role()),
      async assignees => {
        await waitFor(() => expect(assignees.assignees[0]?.label).toBe('ada'));
      },
    );
  });

  it('falls back to the principal id for a user the directory does not carry', async () => {
    getAllUsers.mockResolvedValue(ScyllaResult.success({ items: [] }));

    await inRoot(
      () => createRoleAssignees(() => role()),
      async assignees => {
        await waitFor(() => expect(assignees.assignees[0]?.label).toBe('user-1'));
      },
    );
  });

  it('shows an app principal by its id — the directory only holds users', async () => {
    listGrants.mockResolvedValue(
      ScyllaResult.success([grant({ principal: { kind: PrincipalKind.APP, id: 'app-7' } })]),
    );

    await inRoot(
      () => createRoleAssignees(() => role()),
      async assignees => {
        await waitFor(() => expect(assignees.assignees[0]?.label).toBe('app-7'));
      },
    );
  });

  it('revokes one grant by id', async () => {
    await inRoot(
      () => createRoleAssignees(() => role()),
      async assignees => {
        await waitFor(() => expect(assignees.assignees).toHaveLength(1));

        assignees.remove('grant-1');

        await waitFor(() => expect(revokeGrant).toHaveBeenCalledWith('grant-1'));
      },
    );
  });
});

describe('createGrantTargetLabels', () => {
  it('names a system grant "System", whatever its scope id', async () => {
    await inRoot(
      () => createGrantTargetLabels(() => PermissionScope.SYSTEM),
      targets => {
        expect(targets.labelFor('')).toEqual({ name: 'System', resolved: true });
        expect(targets.labelFor('anything').name).toBe('System');
      },
    );
  });

  it('resolves an organization scope id to its name', async () => {
    await inRoot(
      () => createGrantTargetLabels(() => PermissionScope.ORGANIZATION),
      async targets => {
        await waitFor(() => expect(targets.labelFor('org-1').resolved).toBe(true));
        expect(targets.labelFor('org-1').name).toBe('Acme');
      },
    );
  });

  it('falls back to the raw id, flagged unresolved, while the name is unknown', async () => {
    await inRoot(
      () => createGrantTargetLabels(() => PermissionScope.ORGANIZATION),
      targets => {
        expect(targets.labelFor('org-404')).toEqual({ name: 'org-404', resolved: false });
      },
    );
  });

  it("resolves a project grant to the project, and names the organization owning it", async () => {
    await inRoot(
      () => createGrantTargetLabels(() => PermissionScope.PROJECT),
      async targets => {
        await waitFor(() => expect(targets.labelFor('project-1').resolved).toBe(true));

        expect(targets.labelFor('project-1').name).toBe('Web');
        expect(targets.labelFor('project-1').organizationName).toBe('Acme');
      },
    );
  });

  it('never fans out over the organizations for a non-project scope', async () => {
    await inRoot(
      () => createGrantTargetLabels(() => PermissionScope.ORGANIZATION),
      async targets => {
        await waitFor(() => expect(targets.labelFor('org-1').resolved).toBe(true));

        expect(getProjects).not.toHaveBeenCalled();
      },
    );
  });
});
