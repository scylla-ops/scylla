<script lang="ts">
  import { i18n } from '@lingui/core';
  import { can, Permission, PermissionScope } from '@platform/authz';
  import { contextStore } from '@platform/context';
  import { createQuery } from '@platform/query';
  import { invalidateOrganizationMembers, organizationQueries } from '@/modules/features/organization';
  import { userQueries } from '@/modules/features/user';
  import { ConfirmOperationAlertDialog, FeatureHeader } from '@shared/presentation/ui';
  import { toRune } from '@shared/presentation/stores/to-rune.svelte.ts';
  import { toast } from '@shared/presentation/utils/toast.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { buildOrganizationMembers } from '../../../domain/structs/scope-member.struct.ts';
  import { createAssignableRoles } from '../../assignable-roles.state.svelte.ts';
  import { createScopeMembership } from '../../scope-membership.state.svelte.ts';
  import AddMemberDialog from '../components/AddMemberDialog/AddMemberDialog.svelte';
  import MembersList from '../components/MembersList/MembersList.svelte';
  import { membershipMessages } from '../membership.messages.ts';

  /** The builtin whose whole content is "belongs here, sees it exists". */
  const ORGANIZATION_MEMBER_ROLE_ID = 'organization-member';

  const context = toRune(contextStore);
  const organization = $derived(context().organization);
  const organizationId = $derived(organization.id);
  const target = $derived({ organizationId: organizationId ?? undefined });

  const canManage = $derived(can(Permission.MANAGE_ORG_GRANTS, target));
  // Enumerating accounts is a system capability an org administrator may not
  // hold; without it, roles can still be moved around among people already here.
  const canListUsers = $derived(can(Permission.LIST_USERS));

  const membersQuery = createQuery(() => organizationQueries.members(organizationId));
  const usersQuery = createQuery(() => userQueries.list({ enabled: canListUsers }));

  const members = $derived(membersQuery.data ?? []);

  const roles = createAssignableRoles(PermissionScope.ORGANIZATION);

  const membership = createScopeMembership({
    scope: PermissionScope.ORGANIZATION,
    scopeId: () => organizationId,
    canManage: () => canManage,
    // The member list is derived from grants on the backend, so a grant mutation
    // changes it. The grant mutations cannot reach this key without coupling the
    // two features, which is why the caller invalidates it.
    onMembershipChanged: () => void invalidateOrganizationMembers(organizationId),
  });

  let addOpen = $state(false);
  /** Who is being removed — the id to act on, the name to name in the prompt. */
  let pendingRemoval = $state<{ userId: string; username: string } | null>(null);

  const usernameById = $derived(new Map(members.map(member => [member.userId, member.username])));

  /** The id is the honest fallback: better a raw id than an empty cell. */
  const nameFor = (memberId: string) => usernameById.get(memberId) ?? memberId;

  /**
   * Everyone the organization holds, each with their organization roles. Seeded
   * with the backend's member list so someone reached only through a project is
   * still listed — with no organization role, which is the truth about them.
   */
  const scopeMembers = $derived(
    buildOrganizationMembers(membership.grants, [...usernameById.keys()]),
  );

  /** Only people not already in — re-admitting an existing member is a no-op. */
  const candidates = $derived(
    (usersQuery.data?.items ?? []).filter(user => !usernameById.has(user.userId)),
  );

  const defaultRoleId = $derived(
    roles.assignableRoles.find(role => role.roleId === ORGANIZATION_MEMBER_ROLE_ID)?.roleId,
  );

  const currentUserId = localStorage.getItem('userId') ?? '';

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
  Who belongs to an organization, with which roles, and the operations that
  change either.

  Membership has no storage of its own: the backend derives it from the grants
  table, so this page calls no "add member" or "remove member" RPC — none exists,
  and none is missing. Admitting someone is granting them a role at the
  organization's scope, which is exactly what `AcceptInvitation` does when an
  invitation is accepted; removing them clears every grant they hold at that
  scope *and beneath it*. Both live in `createScopeMembership`.

  Every write needs `MANAGE_ORG_GRANTS` **on this organization** — the scoped
  permission an organization administrator holds, not the system-wide one.
-->
{#snippet blurb()}
  <p class="max-w-3xl text-sm text-muted-foreground">
    {t(membershipMessages.organizationBlurb(organization.name ?? ''))}
  </p>
{/snippet}

{#if !organizationId}
  <div class="flex h-full items-center justify-center">
    <p class="text-sm text-muted-foreground">{t(membershipMessages.selectAnOrganization)}</p>
  </div>
{:else}
  <div class="flex h-full min-h-0 w-full flex-col gap-4">
    <FeatureHeader
      count={scopeMembers.length}
      label={t(membershipMessages.member)}
      pluralLabel={t(membershipMessages.members)}
      underLabel={blurb}
      onNew={() => (addOpen = true)}
      newLabel={t(membershipMessages.addAMember)}
      canNew={canManage}
      newDeniedReason={t(membershipMessages.organizationNewDenied)}
    />

    <MembersList
      members={scopeMembers}
      isLoading={membersQuery.isLoading || membership.isLoading}
      emptyMessage={t(membershipMessages.organizationEmpty)}
      {nameFor}
      labelFor={roles.labelFor}
      {currentUserId}
      {canManage}
      disabled={membership.isPending}
      onRevokeRole={role => void membership.revokeRole(role)}
      addableRolesFor={member =>
        roles.assignableRoles.filter(
          role => !member.roles.some(held => held.roleId === role.roleId),
        )}
      onAddRole={(memberId, roleId) => void membership.addRole(memberId, roleId)}
      emptyRoles={canManage
        ? t(membershipMessages.reachedViaProject)
        : t(membershipMessages.managedByOrgAdmin)}
      canRemove={() => canManage}
      removeTooltip={t(membershipMessages.removeFromOrganization)}
      onRemove={member =>
        (pendingRemoval = { userId: member.userId, username: nameFor(member.userId) })}
    />

    <AddMemberDialog
      open={addOpen}
      onOpenChange={open => (addOpen = open)}
      title={t(membershipMessages.addToOrganization(organization.name ?? ''))}
      description={t(membershipMessages.addToOrganizationBody)}
      {candidates}
      emptyCandidatesLabel={canListUsers
        ? t(membershipMessages.everyoneIsMember)
        : t(membershipMessages.cannotBrowseDirectory)}
      roles={roles.assignableRoles}
      rolesLabel={t(membershipMessages.rolesToGrant)}
      rolesLoading={roles.isLoading}
      isPending={membership.isPending}
      {defaultRoleId}
      onSubmit={handleAdd}
    />

    <ConfirmOperationAlertDialog
      open={pendingRemoval !== null}
      onOpenChange={open => {
        if (!open) pendingRemoval = null;
      }}
      isLoading={membership.isRemoving}
      title={t(
        membershipMessages.confirmRemoveFromOrganization(
          pendingRemoval?.username ?? '',
          organization.name ?? '',
        ),
      )}
      description={t(membershipMessages.confirmRemoveFromOrganizationBody)}
      onContinue={() => void handleRemove()}
    />
  </div>
{/if}
