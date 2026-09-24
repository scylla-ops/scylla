<script lang="ts">
  import { i18n } from '@lingui/core';
  import { can, Permission, PermissionScope } from '@platform/authz';
  import { contextStore } from '@platform/context';
  import { createQuery } from '@platform/query';
  import { organizationQueries } from '@/modules/features/organization';
  import { invalidateProjectMembers, projectQueries } from '@/modules/features/project';
  import { roleConfers, roleQueries } from '@/modules/features/roles';
  import { ConfirmOperationAlertDialog, FeatureHeader } from '@shared/presentation/ui';
  import { toRune } from '@shared/presentation/stores/to-rune.svelte.ts';
  import { toast } from '@shared/presentation/utils/toast.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import {
    buildProjectMembers,
    MemberRoleOrigin,
    type ScopeMember,
  } from '../../domain/structs/scope-member.struct.ts';
  import { createAssignableRoles } from '../assignable-roles.state.svelte.ts';
  import { createScopeMembership } from '../scope-membership.state.svelte.ts';
  import AddMemberDialog from './components/AddMemberDialog.svelte';
  import MembersHint from './components/MembersHint.svelte';
  import MembersList from './components/MembersList.svelte';
  import { membershipMessages } from './membership.messages.ts';

  interface Props {
    /** From the route: `/:organizationSlug/projects/:projectId/members`. */
    projectId?: string;
  }

  let { projectId }: Props = $props();

  const context = toRune(contextStore);
  const organizationId = $derived(context().organization.id);
  // Every check is about *this* project, not whichever one the context store
  // happens to hold — a direct URL hit may land here before the two agree.
  const target = $derived({
    projectId: projectId ?? undefined,
    organizationId: organizationId ?? undefined,
  });

  const canManage = $derived(can(Permission.MANAGE_PROJECT_GRANTS, target));
  // Inherited roles live in the organization's grant list, which only an
  // organization administrator may read. Without it the project half is still
  // complete — the page says what it cannot show instead of implying emptiness.
  const canReadOrganizationGrants = $derived(can(Permission.MANAGE_ORG_GRANTS, target));
  const canListOrganizationMembers = $derived(
    can(Permission.LIST_ORGANIZATION_MEMBERS, target),
  );

  const projectMembersQuery = createQuery(() =>
    projectQueries.members(projectId ?? null, {
      enabled: can(Permission.LIST_PROJECT_MEMBERS, target),
    }),
  );
  const organizationMembersQuery = createQuery(() =>
    organizationQueries.members(organizationId, { enabled: canListOrganizationMembers }),
  );
  const organizationGrantsQuery = createQuery(() =>
    roleQueries.scopedGrants(PermissionScope.ORGANIZATION, organizationId, {
      enabled: canReadOrganizationGrants,
    }),
  );

  const projectMembers = $derived(projectMembersQuery.data ?? []);
  const organizationMembers = $derived(organizationMembersQuery.data ?? []);
  const organizationGrants = $derived(organizationGrantsQuery.data ?? []);

  const roles = createAssignableRoles(PermissionScope.PROJECT);

  const membership = createScopeMembership({
    scope: PermissionScope.PROJECT,
    scopeId: () => projectId ?? null,
    canManage: () => canManage,
    // Members are derived from grants on the backend, so a grant mutation
    // changes the list, and the grant mutations cannot reach this key without
    // coupling the two features.
    onMembershipChanged: () => void invalidateProjectMembers(projectId ?? null),
  });

  let addOpen = $state(false);
  /** Who is being removed — the id to act on, the name to name in the prompt. */
  let pendingRemoval = $state<{ userId: string; username: string } | null>(null);

  const usernameById = $derived.by(() => {
    // Rebuilt whole by the `$derived` and never mutated after it is read, so a
    // reactive collection would only make a throwaway object track dependencies.
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    const names = new Map<string, string>();
    for (const member of organizationMembers) names.set(member.userId, member.username);
    for (const member of projectMembers) names.set(member.userId, member.username);
    return names;
  });

  /** The id is the honest fallback: better a raw id than an empty cell. */
  const nameFor = (memberId: string) => usernameById.get(memberId) ?? memberId;

  /**
   * An organization role reaches this project when it confers reading a project
   * at all: at organization scope that covers every project beneath it. The
   * floor role (`organization-member`) confers nothing here and is left out —
   * listing it would read as project access nobody actually has.
   */
  const reachesProjects = $derived((roleId: string) =>
    roleConfers(roles.roleById.get(roleId), Permission.READ_PROJECT),
  );

  const members = $derived(
    buildProjectMembers(
      membership.grants,
      organizationGrants,
      reachesProjects,
      projectMembers.map(member => member.userId),
    ),
  );

  const memberIds = $derived(new Set(members.map(member => member.userId)));

  /** People the organization has admitted who hold nothing on this project yet. */
  const candidates = $derived(
    organizationMembers.filter(member => !memberIds.has(member.userId)),
  );

  const currentUserId = localStorage.getItem('userId') ?? '';

  /** The project-scoped roles a member holds, which are the editable ones. */
  const directRoleIds = (member: ScopeMember): Set<string> =>
    new Set(
      member.roles
        .filter(role => role.origin === MemberRoleOrigin.DIRECT)
        .map(role => role.roleId),
    );

  const handleAdd = async (userId: string, roleIds: string[]) => {
    const granted = await membership.grantRoles(userId, roleIds);
    if (granted) toast.success(i18n._(membershipMessages.memberAdded(roleIds.length)));
    return granted;
  };

  const handleRemove = async () => {
    if (!pendingRemoval) return;
    await membership.removeMember(pendingRemoval.userId, pendingRemoval.username);
    pendingRemoval = null;
  };
</script>

<!--
  Who works on a project, with which roles, and where each role comes from.

  The list is assembled here rather than read from one call, because no single
  call answers it: `ListProjectMembers` returns the holders of a project-scoped
  grant and stops there, while an organization role covering every project of the
  organization reaches this project just as effectively. Showing only the first
  would make the project look emptier — and more locked down — than it is. So
  each user appears once, carrying both kinds of role, each badged with the scope
  it is bound to.

  Only the project-scoped ones are editable here. An inherited role lives on a
  grant bound to the organization; revoking it from this page would silently
  change someone's access to *every* project, which is why it is shown locked
  with the reason rather than hidden or offered.

  Adding someone is bounded by the backend's tenant rule: a project grant may
  only go to a person the organization has already admitted, so the candidate
  list is the organization's members, never the whole directory.
-->
{#snippet blurb()}
  <p class="max-w-3xl text-sm text-muted-foreground">{t(membershipMessages.projectBlurb)}</p>
{/snippet}

{#if projectId}
  <div class="flex h-full min-h-0 w-full flex-col gap-4">
    <FeatureHeader
      count={members.length}
      label={t(membershipMessages.member)}
      pluralLabel={t(membershipMessages.members)}
      underLabel={blurb}
      onNew={() => (addOpen = true)}
      newLabel={t(membershipMessages.addAMember)}
      canNew={canManage}
      newDeniedReason={t(membershipMessages.projectNewDenied)}
    />

    <MembersList
      {members}
      isLoading={projectMembersQuery.isLoading || membership.isLoading}
      emptyMessage={t(membershipMessages.projectEmpty)}
      {nameFor}
      labelFor={roles.labelFor}
      {currentUserId}
      {canManage}
      disabled={membership.isPending}
      onRevokeRole={role => void membership.revokeRole(role)}
      addableRolesFor={member => {
        // Only project-scoped roles count as held here: someone who inherits
        // "developer" from the organization may still be given it on this
        // project, and that grant is what survives losing the organization role.
        const held = directRoleIds(member);
        return roles.assignableRoles.filter(role => !held.has(role.roleId));
      }}
      onAddRole={(memberId, roleId) => void membership.addRole(memberId, roleId)}
      emptyRoles={t(membershipMessages.managedByProjectAdmin)}
      canRemove={member => canManage && directRoleIds(member).size > 0}
      removeTooltip={t(membershipMessages.removeFromProject)}
      onRemove={member =>
        (pendingRemoval = { userId: member.userId, username: nameFor(member.userId) })}
    />

    <MembersHint>
      {canReadOrganizationGrants
        ? t(membershipMessages.inheritedHintVisible)
        : t(membershipMessages.inheritedHintHidden)}
    </MembersHint>

    <AddMemberDialog
      open={addOpen}
      onOpenChange={open => (addOpen = open)}
      title={t(membershipMessages.addToProject)}
      description={t(membershipMessages.addToProjectBody)}
      {candidates}
      emptyCandidatesLabel={canListOrganizationMembers
        ? t(membershipMessages.everyOrgMemberHere)
        : t(membershipMessages.cannotBrowseOrgMembers)}
      roles={roles.assignableRoles}
      rolesLabel={t(membershipMessages.projectRolesToGrant)}
      rolesLoading={roles.isLoading}
      isPending={membership.isPending}
      onSubmit={handleAdd}
    />

    <ConfirmOperationAlertDialog
      open={pendingRemoval !== null}
      onOpenChange={open => {
        if (!open) pendingRemoval = null;
      }}
      isLoading={membership.isRemoving}
      title={t(membershipMessages.confirmRemoveFromProject(pendingRemoval?.username ?? ''))}
      description={t(membershipMessages.confirmRemoveFromProjectBody)}
      onContinue={() => void handleRemove()}
    />
  </div>
{/if}
