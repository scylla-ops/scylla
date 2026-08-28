import { useCallback, useMemo, useState } from 'react';
import { useParams } from 'react-router-dom';
import { Trans, useLingui } from '@lingui/react/macro';
import { FeatureHeader } from '@shared/presentation/ui';
import { ConfirmOperationAlertDialog } from '@shared/presentation/ui/feedback/ConfirmOperationAlertDialog.tsx';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { roleConfers } from '@/modules/features/permission/domain/entities/role.entity.ts';
import {
  Permission,
  PermissionScope,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';
import {
  buildProjectMembers,
  MemberRoleOrigin,
  type ScopeMember,
} from '@/modules/features/permission/domain/structs/scope-member.struct.ts';
import { useAssignableRoles } from '@/modules/features/permission/presentation/hooks/use-assignable-roles.ts';
import { useAuthorization } from '@/modules/features/permission/presentation/hooks/use-authorization.ts';
import { useScopedGrants } from '@/modules/features/permission/presentation/hooks/use-grants.ts';
import { useScopeMembership } from '@/modules/features/permission/presentation/hooks/use-scope-membership.ts';
import {
  AddMemberDialog,
  MembersHint,
  MembersList,
} from '@/modules/features/permission/presentation/ui/components/members';
import { useOrganizationMembers } from '@/modules/features/organization/presentation/hooks/use-organization-members.ts';
import { useProjectMembers } from '@/modules/features/project/presentation/hooks/use-project-members.ts';

/** Who is being removed — the id to act on, the name to name in the prompt. */
interface PendingRemoval {
  userId: string;
  username: string;
}

/** The project-scoped roles a member holds, which are the editable ones. */
const directRoleIds = (member: ScopeMember): Set<string> =>
  new Set(
    member.roles
      .filter(role => role.origin === MemberRoleOrigin.DIRECT)
      .map(role => role.roleId),
  );

/**
 * Who works on a project, with which roles, and where each role comes from.
 *
 * The list is assembled here rather than read from one call, because no single
 * call answers it: `ListProjectMembers` returns the holders of a project-scoped
 * grant and stops there, while an organization role covering every project of
 * the organization reaches this project just as effectively. Showing only the
 * first would make the project look emptier — and more locked down — than it is.
 * So each user appears once, carrying both kinds of role, each badged with the
 * scope it is bound to.
 *
 * Only the project-scoped ones are editable here. An inherited role lives on a
 * grant bound to the organization; revoking it from this page would silently
 * change someone's access to *every* project, which is why it is shown locked
 * with the reason rather than hidden or offered.
 *
 * Adding someone is bounded by the backend's tenant rule: a project grant may
 * only go to a person the organization has already admitted, so the candidate
 * list is the organization's members, never the whole directory.
 */
export const ProjectMembersPage = () => {
  const { t } = useLingui();
  const { projectId = null } = useParams<{ projectId: string }>();
  const organizationId = useContextStore(state => state.organization.id);
  // Every check is about *this* project, not whichever one the context store
  // happens to hold — a direct URL hit may land here before the two agree.
  const target = { projectId: projectId ?? undefined, organizationId: organizationId ?? undefined };

  const { can } = useAuthorization();
  const canManage = can(Permission.MANAGE_PROJECT_GRANTS, target);
  // Inherited roles live in the organization's grant list, which only an
  // organization administrator may read. Without it the project half is still
  // complete — the page says what it cannot show instead of implying emptiness.
  const canReadOrganizationGrants = can(Permission.MANAGE_ORG_GRANTS, target);
  const canListOrganizationMembers = can(Permission.LIST_ORGANIZATION_MEMBERS, target);

  const {
    members: projectMembers,
    isLoading: membersLoading,
    refetchMembers,
  } = useProjectMembers(projectId, {
    enabled: can(Permission.LIST_PROJECT_MEMBERS, target),
  });
  const { members: organizationMembers } = useOrganizationMembers(organizationId, {
    enabled: canListOrganizationMembers,
  });

  const {
    grants: projectGrants,
    isLoading: grantsLoading,
    isPending,
    isRemoving,
    grantRoles,
    addRole,
    revokeRole,
    removeMember,
  } = useScopeMembership({
    scope: PermissionScope.PROJECT,
    scopeId: projectId,
    canManage,
    onMembershipChanged: refetchMembers,
  });

  const { grants: organizationGrants } = useScopedGrants(
    PermissionScope.ORGANIZATION,
    organizationId,
    { enabled: canReadOrganizationGrants },
  );

  const {
    assignableRoles,
    labelFor,
    roleById,
    isLoading: rolesLoading,
  } = useAssignableRoles(PermissionScope.PROJECT);

  const [addOpen, setAddOpen] = useState(false);
  const [pendingRemoval, setPendingRemoval] = useState<PendingRemoval | null>(null);

  const usernameById = useMemo(() => {
    const names = new Map<string, string>();
    for (const member of organizationMembers) names.set(member.userId, member.username);
    for (const member of projectMembers) names.set(member.userId, member.username);
    return names;
  }, [organizationMembers, projectMembers]);

  /** The id is the honest fallback: better a raw id than an empty cell. */
  const nameFor = useCallback(
    (memberId: string) => usernameById.get(memberId) ?? memberId,
    [usernameById],
  );

  /**
   * An organization role reaches this project when it confers reading a project
   * at all: at organization scope that covers every project beneath it. The
   * floor role (`organization-member`) confers nothing here and is left out —
   * listing it would read as project access nobody actually has.
   */
  const reachesProjects = useMemo(
    () => (roleId: string) => roleConfers(roleById.get(roleId), Permission.READ_PROJECT),
    [roleById],
  );

  const members = useMemo(
    () =>
      buildProjectMembers(
        projectGrants,
        organizationGrants,
        reachesProjects,
        projectMembers.map(member => member.userId),
      ),
    [projectGrants, organizationGrants, reachesProjects, projectMembers],
  );

  const memberIds = useMemo(() => new Set(members.map(member => member.userId)), [members]);

  /** People the organization has admitted who hold nothing on this project yet. */
  const candidates = useMemo(
    () => organizationMembers.filter(member => !memberIds.has(member.userId)),
    [organizationMembers, memberIds],
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

  if (!projectId) return null;

  return (
    <div className='flex h-full min-h-0 w-full flex-col gap-4'>
      <FeatureHeader
        count={members.length}
        label={<Trans>Member</Trans>}
        pluralLabel={<Trans>Members</Trans>}
        underLabel={
          <p className='max-w-3xl text-sm text-muted-foreground'>
            <Trans>
              Everyone with access to this project, whether granted here or through a role on the
              organization. Only project roles can be changed from this page.
            </Trans>
          </p>
        }
        onNew={() => setAddOpen(true)}
        newLabel={<Trans>Add a member</Trans>}
        canNew={canManage}
        newDeniedReason={<Trans>You don't have permission to add members to this project.</Trans>}
      />

      <MembersList
        members={members}
        isLoading={membersLoading || grantsLoading}
        emptyMessage={<Trans>Nobody has access to this project yet.</Trans>}
        nameFor={nameFor}
        labelFor={labelFor}
        currentUserId={currentUserId}
        canManage={canManage}
        disabled={isPending}
        onRevokeRole={role => void revokeRole(role)}
        /* Only project-scoped roles count as held here: someone who inherits
           "developer" from the organization may still be given it on this
           project, and that grant is what survives losing the organization
           role. */
        addableRolesFor={member => {
          const held = directRoleIds(member);
          return assignableRoles.filter(role => !held.has(role.roleId));
        }}
        onAddRole={(memberId, roleId) => void addRole(memberId, roleId)}
        emptyRoles={<Trans>Managed by a project administrator</Trans>}
        /* Nothing to remove from someone who only inherits: their grants live
           on the organization, and clearing them is done there. */
        canRemove={member => canManage && directRoleIds(member).size > 0}
        removeTooltip={<Trans>Remove from the project</Trans>}
        onRemove={member =>
          setPendingRemoval({ userId: member.userId, username: nameFor(member.userId) })
        }
      />

      <MembersHint>
        {canReadOrganizationGrants ? (
          <Trans>
            Roles badged “Organization” are inherited from the organization and are managed there.
          </Trans>
        ) : (
          <Trans>
            Organization administrators also reach this project. Their roles are managed at the
            organization level and are not listed here.
          </Trans>
        )}
      </MembersHint>

      <AddMemberDialog
        open={addOpen}
        onOpenChange={setAddOpen}
        title={<Trans>Add a member to this project</Trans>}
        description={
          <Trans>
            Only members of the organization can be added — a project role granted to anyone else is
            refused.
          </Trans>
        }
        candidates={candidates}
        emptyCandidatesLabel={
          canListOrganizationMembers
            ? t`Every member of the organization is already here`
            : t`You can't browse the organization's members`
        }
        roles={assignableRoles}
        rolesLabel={<Trans>Project roles to grant</Trans>}
        rolesLoading={rolesLoading}
        isPending={isPending}
        onSubmit={handleAdd}
      />

      <ConfirmOperationAlertDialog
        open={!!pendingRemoval}
        onOpenChange={open => {
          if (!open) setPendingRemoval(null);
        }}
        isLoading={isRemoving}
        title={<Trans>Remove {pendingRemoval?.username} from this project?</Trans>}
        description={
          <Trans>
            Every role they hold on this project will be revoked. Roles inherited from the
            organization are untouched — they are managed at the organization level.
          </Trans>
        }
        onContinue={() => void handleRemove()}
      />
    </div>
  );
};

export default ProjectMembersPage;
