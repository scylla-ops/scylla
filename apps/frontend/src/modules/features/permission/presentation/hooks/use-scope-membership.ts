import { useCallback } from 'react';
import { useLingui } from '@lingui/react/macro';
import { toast } from '@shared/presentation/utils/toast.ts';
import {
  PrincipalKind,
  type PermissionScope,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';
import type { MemberRole } from '@/modules/features/permission/domain/structs/scope-member.struct.ts';
import { useScopedGrants } from '@/modules/features/permission/presentation/hooks/use-grants.ts';

interface UseScopeMembershipOptions {
  scope: PermissionScope;
  /** The org/project the grants are bound to; `null` while it is still resolving. */
  scopeId: string | null;
  /** Whether the caller may read and write this scope's grants. */
  canManage: boolean;
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
 */
export const useScopeMembership = ({
  scope,
  scopeId,
  canManage,
  onMembershipChanged,
}: UseScopeMembershipOptions) => {
  const { t } = useLingui();
  const { grants, isLoading, createGrant, revokeGrant, revokeAllAccess } = useScopedGrants(
    scope,
    scopeId,
    { enabled: canManage },
  );

  const isPending = createGrant.isPending || revokeGrant.isPending || revokeAllAccess.isPending;

  /**
   * One grant per role: a grant carries exactly one role, by design. Answers
   * whether the whole batch landed, so the caller can keep its form open on
   * failure — the error itself is already toasted by the mutation cache.
   */
  const grantRoles = useCallback(
    async (userId: string, roleIds: string[]): Promise<boolean> => {
      if (!scopeId || roleIds.length === 0) return false;
      try {
        await Promise.all(
          roleIds.map(roleId =>
            createGrant.mutateAsync({
              principal: { kind: PrincipalKind.USER, id: userId },
              roleId,
              scope,
              scopeId,
            }),
          ),
        );
        onMembershipChanged?.();
        return true;
      } catch {
        // The mutation cache already toasted the failure.
        return false;
      }
    },
    [createGrant, onMembershipChanged, scope, scopeId],
  );

  /** Hands one more role to someone already listed. */
  const addRole = useCallback(
    async (userId: string, roleId: string) => {
      if (await grantRoles(userId, [roleId])) toast.success(t`Role granted`);
    },
    [grantRoles, t],
  );

  const revokeRole = useCallback(
    async (role: MemberRole) => {
      try {
        await revokeGrant.mutateAsync(role.grantId);
        onMembershipChanged?.();
        toast.success(t`Role revoked`);
      } catch {
        // Already toasted globally — the last-owner guard lands here too.
      }
    },
    [onMembershipChanged, revokeGrant, t],
  );

  /** Clears every grant the user holds at this scope and beneath it. */
  const removeMember = useCallback(
    async (userId: string, username: string) => {
      if (!scopeId) return;
      try {
        const revoked = await revokeAllAccess.mutateAsync({
          principal: { kind: PrincipalKind.USER, id: userId },
          scope,
          scopeId,
        });
        onMembershipChanged?.();
        toast.success(t`${username} removed — ${revoked} grant(s) revoked`);
      } catch {
        // Already toasted globally — the last-owner guard lands here too.
      }
    },
    [onMembershipChanged, revokeAllAccess, scope, scopeId, t],
  );

  return {
    grants,
    isLoading,
    /** Any write in flight — what disables the whole view's controls. */
    isPending,
    isRemoving: revokeAllAccess.isPending,
    grantRoles,
    addRole,
    revokeRole,
    removeMember,
  };
};
