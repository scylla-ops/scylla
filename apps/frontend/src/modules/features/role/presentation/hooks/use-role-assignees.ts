import { useMemo } from 'react';
import type { RoleEntity } from '@/modules/features/role/domain/entities/role.entity.ts';
import type { GrantEntity } from '@/modules/features/role/domain/entities/grant.entity.ts';
import { PrincipalKind } from '@/modules/features/role/domain/structs/permission.struct.ts';
import { useGrants } from '@/modules/features/role/presentation/hooks/use-grants.ts';
import { useUsers } from '@/modules/features/user/presentation/hooks/use-users.ts';

export interface RoleAssignee {
  grant: GrantEntity;
  /** Resolved display name (username for users), falling back to the principal id. */
  label: string;
}

/**
 * Assignees of a single role plus the mutation to revoke a grant. Wraps
 * {@link useGrants} and resolves principal ids to usernames. Grant creation
 * lives in the {@link GrantCreator} dialog, which can target any scope.
 */
export const useRoleAssignees = (role: RoleEntity | null) => {
  const { grants, revokeGrant } = useGrants();
  const { users } = useUsers();

  const usernameById = useMemo(
    () => new Map((users?.items ?? []).map(user => [user.userId, user.username])),
    [users],
  );

  const assignees: RoleAssignee[] = useMemo(() => {
    if (!role) return [];
    return grants
      .filter(grant => grant.target.kind === 'role' && grant.target.roleId === role.id)
      .map(grant => ({
        grant,
        label:
          grant.principal.kind === PrincipalKind.USER
            ? (usernameById.get(grant.principal.id) ?? grant.principal.id)
            : grant.principal.id,
      }));
  }, [grants, role, usernameById]);

  const removeAssignee = (grantId: string) => revokeGrant.mutate(grantId);

  return {
    assignees,
    removeAssignee,
    isRemoving: revokeGrant.isPending,
  };
};
