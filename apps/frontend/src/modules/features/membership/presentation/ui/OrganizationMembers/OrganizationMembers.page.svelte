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

  /** The builtin that only means "belongs here". */
  const ORGANIZATION_MEMBER_ROLE_ID = 'organization-member';

  const context = toRune(contextStore);
  const organization = $derived(context().organization);
  const organizationId = $derived(organization.id);
  const target = $derived({ organizationId: organizationId ?? undefined });

  const canManage = $derived(can(Permission.MANAGE_ORG_GRANTS, target));
  // Without LIST_USERS, roles can still be changed among the people already here.
  const canListUsers = $derived(can(Permission.LIST_USERS));

  const membersQuery = createQuery(() => organizationQueries.members(organizationId));
  const usersQuery = createQuery(() => userQueries.list({ enabled: canListUsers }));

  const members = $derived(membersQuery.data ?? []);

  const roles = createAssignableRoles(PermissionScope.ORGANIZATION);

  const membership = createScopeMembership({
    scope: PermissionScope.ORGANIZATION,
    scopeId: () => organizationId,
    canManage: () => canManage,
    // The backend derives members from grants; the grant mutations cannot reach this key.
    onMembershipChanged: () => void invalidateOrganizationMembers(organizationId),
  });

  let addOpen = $state(false);
  let pendingRemoval = $state<{ userId: string; username: string } | null>(null);

  const usernameById = $derived(new Map(members.map(member => [member.userId, member.username])));

  const nameFor = (memberId: string) => usernameById.get(memberId) ?? memberId;

  /** Seeded with the backend's list: someone reached only through a project is listed too. */
  const scopeMembers = $derived(
    buildOrganizationMembers(membership.grants, [...usernameById.keys()]),
  );

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

<!-- Writes need `MANAGE_ORG_GRANTS` on this organization. -->
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
