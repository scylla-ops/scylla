<script lang="ts">
  import LockIcon from '@lucide/svelte/icons/lock';
  import XIcon from '@lucide/svelte/icons/x';
  import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn';
  import { scopeLabelOf } from '@/modules/features/roles';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { MemberRoleOrigin, type MemberRole } from '../../../../domain/structs/scope-member.struct.ts';
  import { membershipMessages } from '../../membership.messages.ts';

  interface Props {
    roles: MemberRole[];
    labelFor: (roleId: string) => string;
    /** Without it, the chips are read-only. */
    canManage?: boolean;
    disabled?: boolean;
    onRevoke?: (role: MemberRole) => void;
    empty?: string;
  }

  let {
    roles,
    labelFor,
    canManage = false,
    disabled = false,
    onRevoke,
    empty,
  }: Props = $props();

  /** Not `Badge`: its fixed height clips the scope dot and the revoke button. */
  const CHIP =
    'inline-flex h-6 max-w-full items-center gap-1.5 rounded-full border py-0.5 pl-2.5 text-xs leading-none';

  const DIRECT_CHIP = 'border-transparent bg-secondary font-medium text-secondary-foreground';

  /** From an enclosing scope, administered there: dashed and locked. */
  const INHERITED_CHIP = 'border-dashed border-border bg-muted/40 text-muted-foreground';

  const isInherited = (role: MemberRole) => role.origin === MemberRoleOrigin.INHERITED;
  const hasTrailing = (role: MemberRole) => isInherited(role) || canManage;
</script>

<!--
  Direct roles can be revoked here; inherited ones are shown locked, not hidden.
  Only inherited chips show their scope: it says where to change them.
-->
{#snippet chip(role: MemberRole)}
  <span
    class={cn(
      CHIP,
      hasTrailing(role) ? 'pr-1.5' : 'pr-2.5',
      isInherited(role) ? INHERITED_CHIP : DIRECT_CHIP,
    )}
  >
    <span class="max-w-[10rem] truncate">{labelFor(role.roleId)}</span>
    {#if isInherited(role)}
      <span
        class="shrink-0 rounded-full bg-background/70 px-1.5 py-0.5 text-[10px] uppercase leading-none tracking-wide"
      >
        {scopeLabelOf(role.scope)}
      </span>
      <LockIcon class="size-3 shrink-0" />
    {:else if canManage}
      <button
        type="button"
        {disabled}
        aria-label={t(membershipMessages.revokeRole(labelFor(role.roleId)))}
        class="inline-flex size-4 shrink-0 items-center justify-center rounded-full transition-colors hover:bg-destructive-subtle hover:text-destructive disabled:pointer-events-none disabled:opacity-50"
        onclick={() => onRevoke?.(role)}
      >
        <XIcon class="size-3" />
      </button>
    {/if}
  </span>
{/snippet}

{#if roles.length === 0}
  <span class="text-xs italic text-muted-foreground">
    {empty ?? t(membershipMessages.noRole)}
  </span>
{:else}
  <div class="flex min-w-0 flex-wrap items-center gap-1.5">
    {#each roles as role (role.grantId)}
      {#if isInherited(role)}
        <Tooltip>
          <TooltipTrigger>
            {#snippet child({ props })}
              <span {...props} class="inline-flex max-w-full">{@render chip(role)}</span>
            {/snippet}
          </TooltipTrigger>
          <TooltipContent>{t(membershipMessages.managedAtOrganization)}</TooltipContent>
        </Tooltip>
      {:else}
        <span>{@render chip(role)}</span>
      {/if}
    {/each}
  </div>
{/if}
