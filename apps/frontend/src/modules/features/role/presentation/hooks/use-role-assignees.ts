import { useMemo } from 'react';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import type { RoleEntity } from '@/modules/features/role/domain/entities/role.entity.ts';
import type { GrantEntity } from '@/modules/features/role/domain/entities/grant.entity.ts';
import {
  PermissionScope,
  PrincipalKind,
} from '@/modules/features/role/domain/structs/permission.struct.ts';
import { useGrants } from '@/modules/features/role/presentation/hooks/use-grants.ts';
import { useUsers } from '@/modules/features/user/presentation/hooks/use-users.ts';

export interface RoleAssignee {
  grant: GrantEntity;
  /** Resolved display name (username for users), falling back to the principal id. */
  label: string;
}

/** The scope id a grant of `role` must bind to, taken from the current context. */
const scopeIdFor = (scope: PermissionScope, orgId: string | null, projectId: string | null) => {
  switch (scope) {
    case PermissionScope.ORGANIZATION:
      return orgId ?? '';
    case PermissionScope.PROJECT:
      return projectId ?? '';
    default:
      return '';
  }
};

/**
 * Assignees of a single role plus the mutations to grant/revoke the role to a
 * user. Wraps {@link useGrants} and resolves principal ids to usernames.
 */
export const useRoleAssignees = (role: RoleEntity | null) => {
  const { grants, createGrant, revokeGrant } = useGrants();
  const { users } = useUsers();
  const orgId = useContextStore(state => state.organization.id);
  const projectId = useContextStore(state => state.project.id);

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

  const assignedUserIds = useMemo(
    () =>
      new Set(
        assignees
          .filter(a => a.grant.principal.kind === PrincipalKind.USER)
          .map(a => a.grant.principal.id),
      ),
    [assignees],
  );

  /** Users not yet holding this role — candidates for the "add" picker. */
  const assignableUsers = useMemo(
    () => (users?.items ?? []).filter(user => !assignedUserIds.has(user.userId)),
    [users, assignedUserIds],
  );

  const addUser = (userId: string) => {
    if (!role) return;
    createGrant.mutate({
      principal: { kind: PrincipalKind.USER, id: userId },
      target: { kind: 'role', roleId: role.id },
      scope: role.scope,
      scopeId: scopeIdFor(role.scope, orgId, projectId),
    });
  };

  const removeAssignee = (grantId: string) => revokeGrant.mutate(grantId);

  return {
    assignees,
    assignableUsers,
    addUser,
    removeAssignee,
    isAdding: createGrant.isPending,
    isRemoving: revokeGrant.isPending,
  };
};
