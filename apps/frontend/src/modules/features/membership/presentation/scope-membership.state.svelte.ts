import { i18n } from '@lingui/core';
import { PrincipalKind, type PermissionScope } from '@platform/authz';
import { createMutation, createQuery } from '@platform/query';
import { grantMutations, roleQueries } from '@/modules/features/roles';
import { toast } from '@shared/presentation/utils/toast.ts';
import type { MemberRole } from '../domain/structs/scope-member.struct.ts';
import { membershipMessages } from './ui/membership.messages.ts';

export interface ScopeMembershipOptions {
  scope: PermissionScope;
  scopeId: () => string | null;
  canManage: () => boolean;
  /** Refreshes the backend's member list, which another feature's query key holds. */
  onMembershipChanged?: () => void;
}

/**
 * The grants of one scope, and the operations that change who holds what. There is
 * no member RPC: admitting is granting a role; removing is `RevokeAllAccess` at the
 * scope and beneath it. `scopeId` and `canManage` are getters: they load after the first render.
 */
export const createScopeMembership = ({
  scope,
  scopeId,
  canManage,
  onMembershipChanged,
}: ScopeMembershipOptions) => {
  const grantsQuery = createQuery(() =>
    roleQueries.scopedGrants(scope, scopeId(), { enabled: canManage() }),
  );

  const createGrant = createMutation(() => grantMutations.create());
  const revokeGrant = createMutation(() => grantMutations.revoke());
  const revokeAllAccess = createMutation(() => grantMutations.revokeAllAccess());

  /** One grant per role. Returns whether all were created, so the form stays open on failure. */
  const grantRoles = async (userId: string, roleIds: string[]): Promise<boolean> => {
    const currentScopeId = scopeId();
    if (!currentScopeId || roleIds.length === 0) return false;

    try {
      await Promise.all(
        roleIds.map(roleId =>
          createGrant.mutateAsync({
            principal: { kind: PrincipalKind.USER, id: userId },
            roleId,
            scope,
            scopeId: currentScopeId,
          }),
        ),
      );
      onMembershipChanged?.();
      return true;
    } catch {
      return false;
    }
  };

  const addRole = async (userId: string, roleId: string) => {
    if (await grantRoles(userId, [roleId])) {
      toast.success(i18n._(membershipMessages.roleGranted));
    }
  };

  const revokeRole = async (role: MemberRole) => {
    try {
      await revokeGrant.mutateAsync(role.grantId);
      onMembershipChanged?.();
      toast.success(i18n._(membershipMessages.roleRevoked));
    } catch {
      // The global mutation handler toasts the error.
    }
  };

  const removeMember = async (userId: string, username: string) => {
    const currentScopeId = scopeId();
    if (!currentScopeId) return;

    try {
      const revoked = await revokeAllAccess.mutateAsync({
        principal: { kind: PrincipalKind.USER, id: userId },
        scope,
        scopeId: currentScopeId,
      });
      onMembershipChanged?.();
      toast.success(i18n._(membershipMessages.memberRemoved(username, revoked)));
    } catch {
      // The global mutation handler toasts the error.
    }
  };

  return {
    get grants() {
      return grantsQuery.data ?? [];
    },
    get isLoading() {
      return grantsQuery.isLoading;
    },
    get isPending() {
      return createGrant.isPending || revokeGrant.isPending || revokeAllAccess.isPending;
    },
    get isRemoving() {
      return revokeAllAccess.isPending;
    },
    grantRoles,
    addRole,
    revokeRole,
    removeMember,
  };
};

export type ScopeMembership = ReturnType<typeof createScopeMembership>;
