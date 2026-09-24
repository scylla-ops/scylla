<script lang="ts">
  import Loader2Icon from '@lucide/svelte/icons/loader-2';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type {
    MemberRole,
    ScopeMember,
  } from '../../../../domain/structs/scope-member.struct.ts';
  import type { AssignableRole } from '../../../assignable-roles.state.svelte.ts';
  import MemberCard from '../MemberCard/MemberCard.svelte';
  import { membershipMessages } from '../../membership.messages.ts';

  interface Props {
    members: ScopeMember[];
    nameFor: (userId: string) => string;
    currentUserId: string;
    isLoading?: boolean;
    emptyMessage?: string;

    labelFor: (roleId: string) => string;
    canManage: boolean;
    disabled: boolean;
    onRevokeRole: (role: MemberRole) => void;
    /** At the view's own scope. */
    addableRolesFor: (member: ScopeMember) => AssignableRole[];
    onAddRole: (userId: string, roleId: string) => void;
    emptyRoles?: string;
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

  /** The grid's footprint, so the page does not jump between states. */
  const PANEL = 'flex flex-1 items-center justify-center rounded-xl border border-dashed p-10';

  /** Sorted here: the backend order is grant insertion. */
  const sorted = $derived(
    [...members].sort((left, right) => nameFor(left.userId).localeCompare(nameFor(right.userId))),
  );
</script>

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
    <!-- Two columns until the screen is wide: a card's width decides how many roles fit per row. -->
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
