<script lang="ts">
  import Loader2Icon from '@lucide/svelte/icons/loader-2';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type {
    MemberRole,
    ScopeMember,
  } from '../../../domain/structs/scope-member.struct.ts';
  import type { AssignableRole } from '../../assignable-roles.state.svelte.ts';
  import MemberCard from './MemberCard.svelte';
  import { membershipMessages } from '../membership.messages.ts';

  interface Props {
    members: ScopeMember[];
    /** Resolves a user id to a display name — falls back to the id upstream. */
    nameFor: (userId: string) => string;
    /** The signed-in user, whose own card is marked instead of removable. */
    currentUserId: string;
    isLoading?: boolean;
    /** Shown in place of the grid when nobody is listed. */
    emptyMessage?: string;

    labelFor: (roleId: string) => string;
    canManage: boolean;
    disabled: boolean;
    onRevokeRole: (role: MemberRole) => void;
    /** Roles this member could still receive at the view's own scope. */
    addableRolesFor: (member: ScopeMember) => AssignableRole[];
    onAddRole: (userId: string, roleId: string) => void;
    /** Shown in the roles box when the member holds none. */
    emptyRoles?: string;
    /** False when this member has nothing removable at this scope. */
    canRemove: (member: ScopeMember) => boolean;
    removeTooltip: string;
    onRemove: (member: ScopeMember) => void;
  }

  let {
    members,
    nameFor,
    currentUserId,
    isLoading = false,
    emptyMessage,
    labelFor,
    canManage,
    disabled,
    onRevokeRole,
    addableRolesFor,
    onAddRole,
    emptyRoles,
    canRemove,
    removeTooltip,
    onRemove,
  }: Props = $props();

  /** Same footprint as the grid, so the page doesn't jump between states. */
  const PANEL = 'flex flex-1 items-center justify-center rounded-xl border border-dashed p-10';

  /**
   * Sorted here rather than by the caller: the order a member list comes back
   * in is grant insertion order, which means nothing to a reader scanning for a
   * person, and both member views want the same answer.
   */
  const sorted = $derived(
    [...members].sort((left, right) => nameFor(left.userId).localeCompare(nameFor(right.userId))),
  );
</script>

<!--
  The member list, as a grid of cards, with the two states that come with it.

  Cards rather than table rows because a member is a small profile — a name and
  a set of role chips — not a row of comparable values: the chips wrap, and in a
  table they either stretch the row or force every column to fight for width.
  Each card owns its own scroll instead (see `MemberCard`).
-->
{#if isLoading}
  <div class="{PANEL} border-border">
    <Loader2Icon role="status" class="size-5 animate-spin text-muted-foreground" />
  </div>
{:else if members.length === 0}
  <div class="{PANEL} border-border">
    <p class="text-center text-sm text-muted-foreground">
      {emptyMessage ?? t(membershipMessages.nobodyListed)}
    </p>
  </div>
{:else}
  <div class="min-h-0 flex-1 overflow-y-auto pr-1">
    <!-- Two columns until the screen is genuinely wide: a card's width is what
         decides how many role chips fit on a row, so a third column bought at
         1280px would cost every card the space the roles need. -->
    <div class="grid grid-cols-1 gap-4 lg:grid-cols-2 2xl:grid-cols-3">
      {#each sorted as member (member.userId)}
        <MemberCard
          name={nameFor(member.userId)}
          roles={member.roles}
          isCurrentUser={member.userId === currentUserId}
          canRemove={canRemove(member)}
          addableRoles={addableRolesFor(member)}
          onAddRole={roleId => onAddRole(member.userId, roleId)}
          onRemove={() => onRemove(member)}
          {labelFor}
          {canManage}
          {disabled}
          {onRevokeRole}
          {emptyRoles}
          {removeTooltip}
        />
      {/each}
    </div>
  </div>
{/if}
