<script lang="ts">
  import { Card } from '@shadcn-svelte';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { MemberRole } from '../../../domain/structs/scope-member.struct.ts';
  import type { AssignableRole } from '../../assignable-roles.state.svelte.ts';
  import AddRoleSelect from './AddRoleSelect.svelte';
  import MemberIdentity from './MemberIdentity.svelte';
  import MemberRoleBadges from './MemberRoleBadges.svelte';
  import MemberRowAction from './MemberRowAction.svelte';
  import { membershipMessages } from '../membership.messages.ts';

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
    /** Shown in the roles box when the member holds none. */
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

<!--
  One member: who they are, the roles they hold here, and the two operations that
  change either.

  Three bands separated by rules — identity, roles, action — rather than one
  padded block, because the three answer different questions and the eye should
  not have to work out where one ends.

  The roles band has a **fixed height and scrolls**, which is the whole reason
  the card can exist. A member's roles are a wrapping set of chips of
  unpredictable count, so letting the band grow would give every card a different
  height and turn the grid into a staircase. Three rows are visible, the count
  under the name says when there are more, and every card lines up.
-->
{#snippet roleCount()}
  {t(membershipMessages.roleCount(roles.length))}
{/snippet}

<Card class="gap-0 overflow-hidden py-0">
  <div class="flex items-center gap-3 px-4 py-3.5">
    <!-- The count is the scroll affordance: it is how a card carrying eight
         roles admits to it while only ever showing three rows of them. -->
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

  <!-- Three chip rows tall, whatever the member holds — see the note above. -->
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
    <!-- A quiet action bar, kept at a constant height: a member with every role
         left to give and one with none must still make cards of the same size. -->
    <div
      class="flex min-h-11 items-center border-t border-border/70 bg-muted/30 px-4 py-2"
    >
      <AddRoleSelect {disabled} roles={addableRoles} onSelect={onAddRole} />
    </div>
  {/if}
</Card>
