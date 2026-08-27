import { useCallback, useMemo, useState } from 'react';
import { Trans, useLingui } from '@lingui/react/macro';
import { FeatureHeader } from '@shared/presentation/ui';
import { ConfirmOperationAlertDialog } from '@shared/presentation/ui/feedback/ConfirmOperationAlertDialog.tsx';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import {
  Permission,
  PermissionScope,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';
import { buildOrganizationMembers } from '@/modules/features/permission/domain/structs/scope-member.struct.ts';
import { useAssignableRoles } from '@/modules/features/permission/presentation/hooks/use-assignable-roles.ts';
import { useAuthorization } from '@/modules/features/permission/presentation/hooks/use-authorization.ts';
import { useScopeMembership } from '@/modules/features/permission/presentation/hooks/use-scope-membership.ts';
import {
  AddMemberDialog,
  MembersHint,
  MembersList,
} from '@/modules/features/permission/presentation/ui/components/members';
import { useOrganizationMembers } from '@/modules/features/organization/presentation/hooks/use-organization-members.ts';
import { useUsers } from '@/modules/features/user/presentation/hooks/use-users.ts';

/** The builtin whose whole content is "belongs here, sees it exists". */
const ORGANIZATION_MEMBER_ROLE_ID = 'organization-member';

/** Who is being removed — the id to act on, the name to name in the prompt. */
interface PendingRemoval {
  userId: string;
  username: string;
}

/**
 * Who belongs to an organization, with which roles, and the operations that
 * change either.
 *
 * Membership has no storage of its own: the backend derives it from the grants
 * table, so this page calls no "add member" or "remove member" RPC — none
 * exists, and none is missing. Admitting someone is granting them a role at the
 * organization's scope, which is exactly what `AcceptInvitation` does when an
 * invitation is accepted; removing them clears every grant they hold at that
 * scope *and beneath it*. Both live in `useScopeMembership`.
 *
 * Every write needs `MANAGE_ORG_GRANTS` **on this organization** — the scoped
 * permission an organization administrator holds, not the system-wide one.
 */
export const OrganizationMembersPage = () => {
  const { t } = useLingui();
  const organization = useContextStore(state => state.organization);
  const organizationId = organization.id;
  const target = { organizationId: organizationId ?? undefined };

  const { can } = useAuthorization();
  const canManage = can(Permission.MANAGE_ORG_GRANTS, target);
  // Enumerating accounts is a system capability an org administrator may not
  // hold; without it, roles can still be moved around among people already here.
  const canListUsers = can(Permission.LIST_USERS);

  const {
    members,
    isLoading: membersLoading,
    refetchMembers,
  } = useOrganizationMembers(organizationId);
  const { users } = useUsers({ enabled: canListUsers });
  const {
    assignableRoles,
    labelFor,
    isLoading: rolesLoading,
  } = useAssignableRoles(PermissionScope.ORGANIZATION);

  const {
    grants,
    isLoading: grantsLoading,
    isPending,
    isRemoving,
    grantRoles,
    addRole,
    revokeRole,
    removeMember,
  } = useScopeMembership({
    scope: PermissionScope.ORGANIZATION,
    scopeId: organizationId,
    canManage,
    onMembershipChanged: refetchMembers,
  });

  const [addOpen, setAddOpen] = useState(false);
  const [pendingRemoval, setPendingRemoval] = useState<PendingRemoval | null>(null);

  const usernameById = useMemo(
    () => new Map(members.map(member => [member.userId, member.username])),
    [members],
  );

  /** The id is the honest fallback: better a raw id than an empty cell. */
  const nameFor = useCallback(
    (memberId: string) => usernameById.get(memberId) ?? memberId,
    [usernameById],
  );

  /**
   * Everyone the organization holds, each with their organization roles. Seeded
   * with the backend's member list so someone reached only through a project is
   * still listed — with no organization role, which is the truth about them.
   */
  const scopeMembers = useMemo(
    () => buildOrganizationMembers(grants, [...usernameById.keys()]),
    [grants, usernameById],
  );

  /** Only people not already in — re-admitting an existing member is a no-op. */
  const candidates = useMemo(
    () => (users?.items ?? []).filter(user => !usernameById.has(user.userId)),
    [users, usernameById],
  );

  const defaultRoleId = useMemo(
    () => assignableRoles.find(role => role.roleId === ORGANIZATION_MEMBER_ROLE_ID)?.roleId,
    [assignableRoles],
  );

  const currentUserId = localStorage.getItem('userId') ?? '';

  const handleAdd = async (userId: string, roleIds: string[]) => {
    const granted = await grantRoles(userId, roleIds);
    if (granted) toast.success(t`Member added with ${roleIds.length} role(s)`);
    return granted;
  };

  const handleRemove = async () => {
    if (!pendingRemoval) return;
    await removeMember(pendingRemoval.userId, pendingRemoval.username);
    setPendingRemoval(null);
  };

  if (!organizationId) {
    return (
      <div className='flex h-full items-center justify-center'>
        <p className='text-sm text-muted-foreground'>
          <Trans>Select an organization to see its members.</Trans>
        </p>
      </div>
    );
  }

  return (
    <div className='flex h-full min-h-0 w-full flex-col gap-4'>
      <FeatureHeader
        count={scopeMembers.length}
        label={<Trans>Member</Trans>}
        pluralLabel={<Trans>Members</Trans>}
        underLabel={
          <p className='max-w-3xl text-sm text-muted-foreground'>
            <Trans>
              Belonging to “{organization.name}” means holding a role in it. Granting someone their
              first role admits them; revoking every role they hold here, and on its projects,
              removes them.
            </Trans>
          </p>
        }
        onNew={() => setAddOpen(true)}
        newLabel={<Trans>Add a member</Trans>}
        canNew={canManage}
        newDeniedReason={<Trans>You don't have permission to admit members here.</Trans>}
      />

      <MembersList
        members={scopeMembers}
        isLoading={membersLoading || grantsLoading}
        emptyMessage={<Trans>Nobody belongs to this organization yet.</Trans>}
        nameFor={nameFor}
        labelFor={labelFor}
        currentUserId={currentUserId}
        canManage={canManage}
        disabled={isPending}
        onRevokeRole={role => void revokeRole(role)}
        addableRolesFor={member =>
          assignableRoles.filter(role => !member.roles.some(held => held.roleId === role.roleId))
        }
        onAddRole={(memberId, roleId) => void addRole(memberId, roleId)}
        emptyRoles={
          canManage ? (
            <Trans>Reached via a project</Trans>
          ) : (
            <Trans>Managed by an organization administrator</Trans>
          )
        }
        canRemove={() => canManage}
        removeTooltip={<Trans>Remove from the organization</Trans>}
        onRemove={member =>
          setPendingRemoval({ userId: member.userId, username: nameFor(member.userId) })
        }
      />

      <AddMemberDialog
        open={addOpen}
        onOpenChange={setAddOpen}
        title={<Trans>Add a member to “{organization.name}”</Trans>}
        description={
          <Trans>Pick someone and the roles they should hold in this organization.</Trans>
        }
        candidates={candidates}
        emptyCandidatesLabel={
          canListUsers ? t`Everyone is already a member` : t`You can't browse the user directory`
        }
        roles={assignableRoles}
        rolesLabel={<Trans>Roles to grant</Trans>}
        rolesLoading={rolesLoading}
        isPending={isPending}
        defaultRoleId={defaultRoleId}
        onSubmit={handleAdd}
      />

      <ConfirmOperationAlertDialog
        open={!!pendingRemoval}
        onOpenChange={open => {
          if (!open) setPendingRemoval(null);
        }}
        isLoading={isRemoving}
        title={
          <Trans>
            Remove {pendingRemoval?.username} from “{organization.name}”?
          </Trans>
        }
        description={
          <Trans>
            Every role they hold on this organization and on its projects will be revoked. They will
            lose access immediately.
          </Trans>
        }
        onContinue={() => void handleRemove()}
      />
    </div>
  );
};

export default OrganizationMembersPage;
