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
  } from '../../../domain/structs/scope-member.struct.ts';
  import { createAssignableRoles } from '../../assignable-roles.state.svelte.ts';
  import { createScopeMembership } from '../../scope-membership.state.svelte.ts';
  import AddMemberDialog from '../components/AddMemberDialog/AddMemberDialog.svelte';
  import MembersHint from '../components/MembersHint/MembersHint.svelte';
  import MembersList from '../components/MembersList/MembersList.svelte';
  import { membershipMessages } from '../membership.messages.ts';

  interface Props {
    projectId?: string;
  }

  let { projectId }: Props = $props();

  const context = toRune(contextStore);
  const organizationId = $derived(context().organization.id);
  // About this project, not the context's: a direct URL can land here before the two agree.
  const target = $derived({
    projectId: projectId ?? undefined,
    organizationId: organizationId ?? undefined,
  });

  const canManage = $derived(can(Permission.MANAGE_PROJECT_GRANTS, target));
  // Inherited roles need the organization's grants (organization admins only).
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
    // The backend derives members from grants; the grant mutations cannot reach this key.
    onMembershipChanged: () => void invalidateProjectMembers(projectId ?? null),
  });

  let addOpen = $state(false);
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

  const nameFor = (memberId: string) => usernameById.get(memberId) ?? memberId;

  /** An organization role reaches the project when it confers reading one. `organization-member` does not. */
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

  const candidates = $derived(
    organizationMembers.filter(member => !memberIds.has(member.userId)),
  );

  const currentUserId = localStorage.getItem('userId') ?? '';

  /** The project's own roles: the editable ones. */
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
  The project's own members plus the organization roles that reach it, each badged
  with its scope. Inherited roles are locked: revoking them changes every project.
  Candidates are the organization's members (a project grant needs one).
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
        // Only project roles count as held: an inherited role can still be granted here.
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
