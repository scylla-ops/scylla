<script lang="ts">
  import { Checkbox as CheckboxPrimitive } from 'bits-ui';
  import CheckIcon from '@lucide/svelte/icons/check';
  import { cn } from '@shared/presentation/utils';

  let { class: className, checked = $bindable(false), ...rest }: CheckboxPrimitive.RootProps =
    $props();
</script>

<CheckboxPrimitive.Root
  data-slot="checkbox"
  bind:checked
  class={cn(
    'peer size-4 shrink-0 rounded-[4px] border border-input shadow-xs transition-shadow outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 data-[state=checked]:border-primary data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground dark:bg-input/30 dark:aria-invalid:ring-destructive/40 dark:data-[state=checked]:bg-primary',
    className,
  )}
  {...rest}
>
  <!--
    Radix had a separate `Indicator` that mounted only when checked; bits-ui
    hands the state to the children snippet instead, so the `{#if}` is what
    replaces it. No indeterminate arm: nothing in the app sets it, and bits-ui's
    `indeterminate` prop still rides through `rest` for the day one does.
  -->
  {#snippet children({ checked: isChecked })}
    <span
      data-slot="checkbox-indicator"
      class="grid place-content-center text-current transition-none"
    >
      {#if isChecked}
        <CheckIcon class="size-3.5" />
      {/if}
    </span>
  {/snippet}
</CheckboxPrimitive.Root>
