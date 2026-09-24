import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { PermissionScope, PrincipalKind, permissionsStore } from '@platform/authz';
import { withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { MemberRoleOrigin } from '../../domain/structs/scope-member.struct.ts';
import type { MemberRole } from '../../domain/structs/scope-member.struct.ts';
import { createScopeMembership } from '../scope-membership.state.svelte.ts';

vi.mock('@shared/presentation/utils/toast.ts', () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

const toastSuccess = vi.mocked(toast.success);

let createGrant: ReturnType<typeof vi.fn>;
let revokeGrant: ReturnType<typeof vi.fn>;
let revokeAllAccess: ReturnType<typeof vi.fn>;
let cache: ReturnType<typeof withQueryClient>;
let restoreRegistry: () => void;

beforeEach(() => {
  vi.mocked(toastSuccess).mockClear();

  createGrant = vi.fn().mockResolvedValue(ScyllaResult.success(undefined));
  revokeGrant = vi.fn().mockResolvedValue(ScyllaResult.success(undefined));
  revokeAllAccess = vi.fn().mockResolvedValue(ScyllaResult.success(3));

  cache = withQueryClient();
  restoreRegistry = withRegistry({
    roles: {
      permissionRepository: {
        createGrant,
        revokeGrant,
        revokeAllAccess,
        listGrants: vi.fn().mockResolvedValue(ScyllaResult.success([])),
        getMyPermissions: vi.fn().mockResolvedValue(ScyllaResult.success({ scopes: [] })),
      },
    },
  });
});

afterEach(() => {
  cache.restore();
  restoreRegistry();
  permissionsStore.setState({ permissions: null });
});

/** Builds the ViewModel inside a reactive root — it is made of runes. */
const withMembership = async (
  overrides: { scopeId?: string | null; onMembershipChanged?: () => void } = {},
  body: (state: ReturnType<typeof createScopeMembership>) => Promise<void> | void = () => {},
) => {
  const { scopeId = 'project-1', onMembershipChanged = vi.fn() } = overrides;
  let state!: ReturnType<typeof createScopeMembership>;

  const cleanup = $effect.root(() => {
    state = createScopeMembership({
      scope: PermissionScope.PROJECT,
      scopeId: () => scopeId,
      canManage: () => true,
      onMembershipChanged,
    });
  });

  try {
    await body(state);
  } finally {
    cleanup();
  }
  return { onMembershipChanged };
};

const memberRole = (overrides: Partial<MemberRole> = {}): MemberRole =>
  ({
    grantId: 'grant-1',
    roleId: 'role-a',
    origin: MemberRoleOrigin.DIRECT,
    ...overrides,
  }) as MemberRole;

describe('grantRoles', () => {
  it('creates one grant per role id, each scoped to this view', async () => {
    const { onMembershipChanged } = await withMembership({}, async state => {
      await expect(state.grantRoles('user-1', ['role-a', 'role-b'])).resolves.toBe(true);
    });

    expect(createGrant).toHaveBeenCalledWith({
      principal: { kind: PrincipalKind.USER, id: 'user-1' },
      roleId: 'role-a',
      scope: PermissionScope.PROJECT,
      scopeId: 'project-1',
    });
    expect(createGrant).toHaveBeenCalledWith(expect.objectContaining({ roleId: 'role-b' }));
    expect(onMembershipChanged).toHaveBeenCalled();
  });

  it('is a no-op returning false while the scope id is still resolving', async () => {
    const { onMembershipChanged } = await withMembership({ scopeId: null }, async state => {
      await expect(state.grantRoles('user-1', ['role-a'])).resolves.toBe(false);
    });

    expect(createGrant).not.toHaveBeenCalled();
    expect(onMembershipChanged).not.toHaveBeenCalled();
  });

  it('refuses an empty role list rather than issuing a pointless round trip', async () => {
    await withMembership({}, async state => {
      await expect(state.grantRoles('user-1', [])).resolves.toBe(false);
    });

    expect(createGrant).not.toHaveBeenCalled();
  });

  it('answers false when a grant fails, so the caller can keep its form open', async () => {
    createGrant.mockRejectedValue(new Error('denied'));

    const { onMembershipChanged } = await withMembership({}, async state => {
      await expect(state.grantRoles('user-1', ['role-a'])).resolves.toBe(false);
    });

    expect(onMembershipChanged).not.toHaveBeenCalled();
  });
});

describe('addRole', () => {
  it('grants the single role and confirms it', async () => {
    await withMembership({}, async state => {
      await state.addRole('user-1', 'role-a');
    });

    expect(createGrant).toHaveBeenCalledWith(expect.objectContaining({ roleId: 'role-a' }));
    expect(toastSuccess).toHaveBeenCalled();
  });

  it('stays silent when the grant failed — the failure is already toasted globally', async () => {
    createGrant.mockRejectedValue(new Error('denied'));

    await withMembership({}, async state => {
      await state.addRole('user-1', 'role-a');
    });

    expect(toastSuccess).not.toHaveBeenCalled();
  });
});

describe('revokeRole', () => {
  it('revokes the one grant by id and refreshes the member list', async () => {
    const { onMembershipChanged } = await withMembership({}, async state => {
      await state.revokeRole(memberRole({ grantId: 'grant-9' }));
    });

    expect(revokeGrant).toHaveBeenCalledWith('grant-9');
    expect(onMembershipChanged).toHaveBeenCalled();
    expect(toastSuccess).toHaveBeenCalled();
  });

  it('swallows a refused revoke — the last-owner guard lands here', async () => {
    revokeGrant.mockRejectedValue(new Error('last owner'));

    const { onMembershipChanged } = await withMembership({}, async state => {
      await expect(state.revokeRole(memberRole())).resolves.toBeUndefined();
    });

    expect(onMembershipChanged).not.toHaveBeenCalled();
    expect(toastSuccess).not.toHaveBeenCalled();
  });
});

describe('removeMember', () => {
  it('clears every grant at this scope and beneath, not just the scope’s own', async () => {
    const { onMembershipChanged } = await withMembership({}, async state => {
      await state.removeMember('user-1', 'alice');
    });

    // RevokeAllAccess, never a series of revokeGrant: narrower grants left
    // behind would keep the person listed.
    expect(revokeAllAccess).toHaveBeenCalledWith({
      principal: { kind: PrincipalKind.USER, id: 'user-1' },
      scope: PermissionScope.PROJECT,
      scopeId: 'project-1',
    });
    expect(revokeGrant).not.toHaveBeenCalled();
    expect(onMembershipChanged).toHaveBeenCalled();
  });

  it('does nothing while the scope id is still resolving', async () => {
    await withMembership({ scopeId: null }, async state => {
      await state.removeMember('user-1', 'alice');
    });

    expect(revokeAllAccess).not.toHaveBeenCalled();
  });

  it('swallows a refused removal', async () => {
    revokeAllAccess.mockRejectedValue(new Error('last owner'));

    const { onMembershipChanged } = await withMembership({}, async state => {
      await expect(state.removeMember('user-1', 'alice')).resolves.toBeUndefined();
    });

    expect(onMembershipChanged).not.toHaveBeenCalled();
    expect(toastSuccess).not.toHaveBeenCalled();
  });
});
