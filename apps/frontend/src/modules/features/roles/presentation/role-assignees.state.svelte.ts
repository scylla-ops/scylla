import { PrincipalKind } from '@platform/authz';
import { createMutation, createQuery } from '@platform/query';
import { userQueries } from '@/modules/features/user';
import type { GrantEntity } from '../domain/entities/grant.entity.ts';
import type { RoleEntity } from '../domain/entities/role.entity.ts';
import { grantMutations, roleQueries } from './roles.queries.ts';

export interface RoleAssignee {
  grant: GrantEntity;
  /** The username, or the principal id. */
  label: string;
}

/** Who holds a role, and revoking it. Creating grants is the grant dialog's job. */
export const createRoleAssignees = (role: () => RoleEntity) => {
  const grantsQuery = createQuery(() => roleQueries.allGrants());
  const usersQuery = createQuery(() => userQueries.list());
  const revokeGrant = createMutation(() => grantMutations.revoke());

  const usernameById = $derived(
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    new Map((usersQuery.data?.items ?? []).map(user => [user.userId, user.username])),
  );

  const assignees = $derived.by((): RoleAssignee[] =>
    (grantsQuery.data ?? [])
      .filter(grant => grant.roleId === role().id)
      .map(grant => ({
        grant,
        label:
          grant.principal.kind === PrincipalKind.USER
            ? (usernameById.get(grant.principal.id) ?? grant.principal.id)
            : grant.principal.id,
      })),
  );

  return {
    get assignees() {
      return assignees;
    },
    get isRemoving() {
      return revokeGrant.isPending;
    },
    remove: (grantId: string) => revokeGrant.mutate(grantId),
  };
};

export type RoleAssignees = ReturnType<typeof createRoleAssignees>;
