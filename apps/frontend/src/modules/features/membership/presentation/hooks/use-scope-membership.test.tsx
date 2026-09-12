import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { ReactNode } from 'react';
import { renderHook } from '@testing-library/react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { PermissionScope, PrincipalKind } from '@platform/authz';
import { useScopeMembership } from './use-scope-membership';
import { MemberRoleOrigin } from '@/modules/features/membership/domain/structs/scope-member.struct.ts';
import type * as RolesModule from '@/modules/features/roles';

const toastSuccess = vi.fn();
vi.mock('sonner', () => ({
  toast: { success: (...args: unknown[]) => toastSuccess(...args) },
}));

const createGrantMutateAsync = vi.fn().mockResolvedValue(undefined);
const revokeGrantMutateAsync = vi.fn().mockResolvedValue(undefined);
const revokeAllAccessMutateAsync = vi.fn().mockResolvedValue(3);

vi.mock('@/modules/features/roles', async importOriginal => {
  const actual = await importOriginal<typeof RolesModule>();
  return {
    ...actual,
    useScopedGrants: () => ({
      grants: [],
      isLoading: false,
      createGrant: { mutateAsync: createGrantMutateAsync, isPending: false },
      revokeGrant: { mutateAsync: revokeGrantMutateAsync, isPending: false },
      revokeAllAccess: { mutateAsync: revokeAllAccessMutateAsync, isPending: false },
    }),
  };
});

const Wrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider i18n={i18n}>{children}</I18nProvider>
);

beforeEach(() => {
  toastSuccess.mockClear();
  createGrantMutateAsync.mockClear().mockResolvedValue(undefined);
  revokeGrantMutateAsync.mockClear().mockResolvedValue(undefined);
  revokeAllAccessMutateAsync.mockClear().mockResolvedValue(3);
});

const setup = (onMembershipChanged = vi.fn()) =>
  renderHook(
    () =>
      useScopeMembership({
        scope: PermissionScope.PROJECT,
        scopeId: 'project-1',
        canManage: true,
        onMembershipChanged,
      }),
    { wrapper: Wrapper },
  );

describe('grantRoles', () => {
  it('creates one grant per role id, in parallel, scoped correctly', async () => {
    const onMembershipChanged = vi.fn();
    const { result } = setup(onMembershipChanged);

    const ok = await result.current.grantRoles('user-1', ['role-a', 'role-b']);

    expect(ok).toBe(true);
    expect(createGrantMutateAsync).toHaveBeenCalledWith({
      principal: { kind: PrincipalKind.USER, id: 'user-1' },
      roleId: 'role-a',
      scope: PermissionScope.PROJECT,
      scopeId: 'project-1',
    });
    expect(createGrantMutateAsync).toHaveBeenCalledWith(
      expect.objectContaining({ roleId: 'role-b' }),
    );
    expect(onMembershipChanged).toHaveBeenCalled();
  });

  it('is a no-op returning false when scopeId is not yet resolved', async () => {
    const { result } = renderHook(
      () =>
        useScopeMembership({
          scope: PermissionScope.PROJECT,
          scopeId: null,
          canManage: true,
        }),
      { wrapper: Wrapper },
    );

    expect(await result.current.grantRoles('user-1', ['role-a'])).toBe(false);
    expect(createGrantMutateAsync).not.toHaveBeenCalled();
  });

  it('is a no-op returning false when given no role ids', async () => {
    const { result } = setup();
    expect(await result.current.grantRoles('user-1', [])).toBe(false);
    expect(createGrantMutateAsync).not.toHaveBeenCalled();
  });

  it('returns false and does not call onMembershipChanged when a grant call fails (already toasted by the mutation cache)', async () => {
    createGrantMutateAsync.mockRejectedValueOnce(new Error('denied'));
    const onMembershipChanged = vi.fn();
    const { result } = setup(onMembershipChanged);

    expect(await result.current.grantRoles('user-1', ['role-a'])).toBe(false);
    expect(onMembershipChanged).not.toHaveBeenCalled();
  });
});

describe('addRole', () => {
  it('grants the single role and toasts on success', async () => {
    const { result } = setup();
    await result.current.addRole('user-1', 'role-a');
    expect(toastSuccess).toHaveBeenCalledWith('Role granted');
  });

  it('does not toast when the grant fails', async () => {
    createGrantMutateAsync.mockRejectedValueOnce(new Error('denied'));
    const { result } = setup();
    await result.current.addRole('user-1', 'role-a');
    expect(toastSuccess).not.toHaveBeenCalled();
  });
});

describe('revokeRole', () => {
  it('revokes by grantId, notifies, and toasts', async () => {
    const onMembershipChanged = vi.fn();
    const { result } = setup(onMembershipChanged);

    await result.current.revokeRole({
      grantId: 'grant-1',
      roleId: 'role-a',
      origin: MemberRoleOrigin.DIRECT,
      scope: PermissionScope.PROJECT,
    });

    expect(revokeGrantMutateAsync).toHaveBeenCalledWith('grant-1');
    expect(onMembershipChanged).toHaveBeenCalled();
    expect(toastSuccess).toHaveBeenCalledWith('Role revoked');
  });

  it('swallows a failure (e.g. the last-owner guard) without throwing or toasting locally', async () => {
    revokeGrantMutateAsync.mockRejectedValueOnce(new Error('cannot revoke last owner'));
    const { result } = setup();

    await expect(
      result.current.revokeRole({
        grantId: 'grant-1',
        roleId: 'role-a',
        origin: MemberRoleOrigin.DIRECT,
        scope: PermissionScope.PROJECT,
      }),
    ).resolves.toBeUndefined();
    expect(toastSuccess).not.toHaveBeenCalled();
  });
});

describe('removeMember', () => {
  it('revokes all access, notifies, and toasts with the username and the number of grants revoked', async () => {
    const onMembershipChanged = vi.fn();
    const { result } = setup(onMembershipChanged);

    await result.current.removeMember('user-1', 'ravenne');

    expect(revokeAllAccessMutateAsync).toHaveBeenCalledWith({
      principal: { kind: PrincipalKind.USER, id: 'user-1' },
      scope: PermissionScope.PROJECT,
      scopeId: 'project-1',
    });
    expect(onMembershipChanged).toHaveBeenCalled();
    expect(toastSuccess).toHaveBeenCalledWith('ravenne removed — 3 grant(s) revoked');
  });

  it('is a no-op when scopeId is not yet resolved', async () => {
    const { result } = renderHook(
      () =>
        useScopeMembership({ scope: PermissionScope.PROJECT, scopeId: null, canManage: true }),
      { wrapper: Wrapper },
    );

    await result.current.removeMember('user-1', 'ravenne');
    expect(revokeAllAccessMutateAsync).not.toHaveBeenCalled();
  });

  it('swallows a failure without throwing or toasting', async () => {
    revokeAllAccessMutateAsync.mockRejectedValueOnce(new Error('cannot revoke last owner'));
    const { result } = setup();

    await expect(result.current.removeMember('user-1', 'ravenne')).resolves.toBeUndefined();
    expect(toastSuccess).not.toHaveBeenCalled();
  });
});

describe('isPending / isRemoving', () => {
  it('isPending is false when nothing is in flight', () => {
    const { result } = setup();
    expect(result.current.isPending).toBe(false);
    expect(result.current.isRemoving).toBe(false);
  });
});
