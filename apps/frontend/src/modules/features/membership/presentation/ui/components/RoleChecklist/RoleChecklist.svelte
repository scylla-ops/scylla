<script lang="ts">
  import { Checkbox, Label } from '@shadcn';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { AssignableRole } from '../../../assignable-roles.state.svelte.ts';
  import { membershipMessages } from '../../membership.messages.ts';

  interface Props {
    label: string;
    roles: AssignableRole[];
    isLoading?: boolean;
    disabled?: boolean;
    selected: Set<string>;
    /** Ticked, locked, labelled. */
    alreadyHeld?: Set<string>;
    onToggle: (roleId: string) => void;
  }

  let {
    label,
    roles,
    isLoading = false,
    disabled = false,
    selected,
    alreadyHeld,
    onToggle,
  }: Props = $props();
</script>

<!-- Not `ScrollArea`: its root does not clip under a `max-h`. -->
<div class="flex min-w-0 flex-col gap-1.5">
  <Label>{label}</Label>
  {#if isLoading}
    <p class="py-4 text-center text-sm text-muted-foreground">
      {t(membershipMessages.loading)}
    </p>
  {:else if roles.length === 0}
    <p
      class="rounded-lg border border-dashed border-border py-4 text-center text-sm text-muted-foreground"
    >
      {t(membershipMessages.noRoleGrantable)}
    </p>
  {:else}
    <div class="max-h-48 overflow-y-auto rounded-lg border border-border p-1">
      <div class="flex flex-col">
        {#each roles as role (role.roleId)}
          {@const held = alreadyHeld?.has(role.roleId) ?? false}
          <label
            class="flex cursor-pointer items-start gap-2 rounded-md px-2 py-1.5 hover:bg-accent/50 aria-disabled:cursor-not-allowed aria-disabled:opacity-60"
            aria-disabled={held || disabled}
          >
            <Checkbox
              class="mt-0.5 shrink-0"
              checked={held || selected.has(role.roleId)}
              disabled={held || disabled}
              onCheckedChange={() => onToggle(role.roleId)}
            />
            <span class="flex min-w-0 flex-1 flex-col gap-0.5">
              <span class="truncate text-sm leading-tight">{role.name}</span>
              {#if role.description}
                <span class="text-xs leading-snug text-muted-foreground">{role.description}</span>
              {/if}
            </span>
            {#if held}
              <span
                class="shrink-0 self-center text-[10px] uppercase tracking-wide text-muted-foreground"
              >
                {t(membershipMessages.held)}
              </span>
            {/if}
          </label>
        {/each}
      </div>
    </div>
  {/if}
</div>
