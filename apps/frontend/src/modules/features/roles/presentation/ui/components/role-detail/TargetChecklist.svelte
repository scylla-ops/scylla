<script lang="ts">
  import { Badge, Checkbox, Label } from '@shadcn-svelte';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { TargetOption } from '../../../grant-creator.state.svelte.ts';
  import { rolesMessages } from '../../roles.messages.ts';

  interface Props {
    label: string;
    /** Shown when the list came back empty — not while it is still loading. */
    empty: string;
    isLoading: boolean;
    options: TargetOption[];
    disabled: boolean;
    isSelected: (id: string) => boolean;
    /** Already held: offered ticked and locked, so the state is visible. */
    isGranted: (id: string) => boolean;
    onToggle: (option: TargetOption) => void;
  }

  let { label, empty, isLoading, options, disabled, isSelected, isGranted, onToggle }: Props =
    $props();
</script>

<div class="flex flex-col gap-1.5">
  <Label>{label}</Label>
  {#if isLoading}
    <p class="py-4 text-center text-sm text-muted-foreground">{t(rolesMessages.loading)}</p>
  {:else if options.length === 0}
    <p class="rounded-lg border border-dashed py-4 text-center text-sm text-muted-foreground">
      {empty}
    </p>
  {:else}
    <!-- Plain overflow rather than the shared `ScrollArea` — see the note in
         `RoleDetailGrantList.svelte`: that root clips nothing under a `max-h`. -->
    <div class="max-h-48 overflow-y-auto rounded-lg border p-2">
      <div class="flex flex-col gap-0.5">
        {#each options as option (option.id)}
          {@const granted = isGranted(option.id)}
          <label
            class="flex cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 hover:bg-secondary aria-disabled:cursor-not-allowed aria-disabled:opacity-60"
            aria-disabled={granted || disabled}
          >
            <Checkbox
              checked={granted || isSelected(option.id)}
              disabled={granted || disabled}
              aria-label={option.name}
              onCheckedChange={() => onToggle(option)}
            />
            <span class="truncate text-sm">{option.name}</span>
            {#if granted}
              <Badge variant="outline" class="ml-auto text-[10px]">
                {t(rolesMessages.granted)}
              </Badge>
            {/if}
          </label>
        {/each}
      </div>
    </div>
  {/if}
</div>
