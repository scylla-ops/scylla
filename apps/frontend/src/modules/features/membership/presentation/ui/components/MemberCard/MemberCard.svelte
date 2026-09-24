<script lang="ts">
  import { Card } from '@shadcn';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { MemberRole } from '../../../../domain/structs/scope-member.struct.ts';
  import type { AssignableRole } from '../../../assignable-roles.state.svelte.ts';
  import AddRoleSelect from '../AddRoleSelect/AddRoleSelect.svelte';
  import MemberIdentity from '../MemberIdentity/MemberIdentity.svelte';
  import MemberRoleBadges from '../MemberRoleBadges/MemberRoleBadges.svelte';
  import MemberRowAction from '../MemberRowAction/MemberRowAction.svelte';
  import { membershipMessages } from '../../membership.messages.ts';

  interface Props {
    name: string;
    roles: MemberRole[];
    isCurrentUser: boolean;
    canRemove: boolean;
    addableRoles: AssignableRole[];
    onAddRole: (roleId: string) => void;
    labelFor: (roleId: string) => string;
    canManage: boolean;
    disabled: boolean;
    onRevokeRole: (role: MemberRole) => void;
    emptyRoles?: string;
    removeTooltip: string;
    onRemove: () => void;
  }

  let {
    name,
    roles,
    isCurrentUser,
    canRemove,
    addableRoles,
    onAddRole,
    labelFor,
    canManage,
    disabled,
    onRevokeRole,
    emptyRoles,
    removeTooltip,
    onRemove,
  }: Props = $props();
</script>

<!-- The roles band has a fixed height and scrolls, so every card has the same height. -->
{#snippet roleCount()}
  {t(membershipMessages.roleCount(roles.length))}
{/snippet}

<Card class="gap-0 overflow-hidden py-0">
  <div class="flex items-center gap-3 px-4 py-3.5">
    <!-- The count tells when there are more roles than the band shows. -->
    <MemberIdentity {name} subtitle={roleCount} />
    <span class="ml-auto shrink-0">
      <MemberRowAction
        {isCurrentUser}
        {canRemove}
        {disabled}
        tooltip={removeTooltip}
        {onRemove}
      />
    </span>
  </div>

  <div class="h-28 overflow-y-auto border-t border-border/70 px-4 py-3">
    <MemberRoleBadges
      {roles}
      {labelFor}
      {canManage}
      {disabled}
      onRevoke={onRevokeRole}
      empty={emptyRoles}
    />
  </div>

  {#if canManage}
    <!-- A constant height, so every card has the same size. -->
    <div
      class="flex min-h-11 items-center border-t border-border/70 bg-muted/30 px-4 py-2"
    >
      <AddRoleSelect {disabled} roles={addableRoles} onSelect={onAddRole} />
    </div>
  {/if}
</Card>
