import { i18n } from '@lingui/core';
import { PrincipalKind, type PermissionScope } from '@platform/authz';
import { createMutation, createQuery } from '@platform/query';
import { grantMutations, roleQueries } from '@/modules/features/roles';
import { toast } from '@shared/presentation/utils/toast.ts';
import type { MemberRole } from '../domain/structs/scope-member.struct.ts';
import { membershipMessages } from './ui/membership.messages.ts';

export interface ScopeMembershipOptions {
  scope: PermissionScope;
  /** The org/project the grants are bound to; `null` while it is still resolving. */
  scopeId: () => string | null;
  /** Whether the caller may read and write this scope's grants. */
  canManage: () => boolean;
  /**
   * Refreshes the backend's own member list. Membership is derived from grants
   * server-side, but that list lives under another feature's query key, so the
   * grant mutations cannot invalidate it themselves.
   */
  onMembershipChanged?: () => void;
}

/**
 * The write half of a member view: the grants bound to one scope, plus the three
 * operations that change who holds what.
 *
 * Membership has no storage of its own — the backend derives it from the grants
 * table — so there is no "add member" or "remove member" RPC to call. Admitting
 * someone *is* granting them a role at the scope; removing them is clearing
 * every grant they hold at that scope and beneath it, which is why the removal
 * goes through `RevokeAllAccess` rather than a series of `RevokeGrant`: revoking
 * only the scope's own grants would leave narrower ones behind, inert but still
 * enough to keep the person listed.
 *
 * Both the organization and the project view need exactly this, with only the
 * scope differing, so it lives here rather than twice in the two pages.
 *
 * `scopeId` and `canManage` are **getters**, not values. The scope id arrives
 * from the route or the context store and the permission from a store that
 * loads after the first paint; capturing either once would freeze the view on
 * whatever the first render happened to hold. The React hook re-ran on every
 * render and got this for free.
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

  /**
   * One grant per role: a grant carries exactly one role, by design. Answers
   * whether the whole batch landed, so the caller can keep its form open on
   * failure — the error itself is already toasted by the mutation cache.
   */
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
      // The mutation cache already toasted the failure.
      return false;
    }
  };

  /** Hands one more role to someone already listed. */
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
      // Already toasted globally — the last-owner guard lands here too.
    }
  };

  /** Clears every grant the user holds at this scope and beneath it. */
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
      // Already toasted globally — the last-owner guard lands here too.
    }
  };

  return {
    get grants() {
      return grantsQuery.data ?? [];
    },
    get isLoading() {
      return grantsQuery.isLoading;
    },
    /** Any write in flight — what disables the whole view's controls. */
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
