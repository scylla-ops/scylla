<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Select as SelectPrimitive } from 'bits-ui';
  import CheckIcon from '@lucide/svelte/icons/check';
  import { cn } from '@shared/presentation/utils';

  type Props = SelectPrimitive.ItemProps & { children?: Snippet };

  // Renamed on destructure: the primitive's own `children` snippet is declared
  // below, and the two would shadow each other under one name.
  let { class: className, children: label, ...rest }: Props = $props();
</script>

<SelectPrimitive.Item
  data-slot="select-item"
  class={cn(
    "focus:bg-accent focus:text-accent-foreground data-highlighted:bg-accent data-highlighted:text-accent-foreground [&_svg:not([class*='text-'])]:text-muted-foreground relative flex w-full cursor-default items-center gap-2 rounded-sm py-1.5 pr-8 pl-2 text-sm outline-hidden select-none data-[disabled]:pointer-events-none data-[disabled]:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
    className,
  )}
  {...rest}
>
  <!--
    Radix had an `ItemIndicator` that mounted only when selected; bits-ui hands
    the state to the children snippet instead, so the `{#if}` replaces it.
    `data-highlighted` joins `focus:` in the class list for a related reason:
    bits-ui never moves DOM focus into the list, it tracks the active item with
    `aria-activedescendant`, so `focus:` alone would highlight nothing while
    arrowing through the options.
  -->
  {#snippet children({ selected })}
    <span class="absolute right-2 flex size-3.5 items-center justify-center">
      {#if selected}
        <CheckIcon class="size-4" />
      {/if}
    </span>
    {@render label?.()}
  {/snippet}
</SelectPrimitive.Item>
