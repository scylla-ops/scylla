<script lang="ts">
  import type { Snippet } from 'svelte';
  import { ToggleGroup as ToggleGroupPrimitive } from 'bits-ui';
  import { cn } from '@shared/presentation/utils';
  import {
    getToggleGroupStyle,
    toggleVariants,
    type ToggleSize,
    type ToggleVariant,
  } from './toggle-variants.ts';

  // `children` is overridden, not inherited: bits-ui types it as a snippet
  // taking the pressed state, and nothing here needs it — the styling reads
  // `data-state`. Callers pass a plain `{label}`.
  type Props = Omit<ToggleGroupPrimitive.ItemProps, 'children'> & {
    variant?: ToggleVariant;
    size?: ToggleSize;
    children?: Snippet;
  };

  // Renamed on destructure: bits-ui's own `children` takes the pressed state as
  // an argument, and the two would shadow each other under one name. Nothing
  // here needs that state — the styling reads `data-state` — so the caller's
  // snippet stays parameterless.
  let { class: className, variant, size, children: label, ...rest }: Props = $props();

  const groupStyle = getToggleGroupStyle();

  // The group wins over the item's own props, as in the React original: the
  // whole point of a segmented control is that its segments match.
  const style = $derived(groupStyle?.() ?? { variant, size, spacing: 0 });
</script>

<ToggleGroupPrimitive.Item
  data-slot="toggle-group-item"
  data-variant={style.variant ?? variant}
  data-size={style.size ?? size}
  data-spacing={style.spacing}
  class={cn(
    toggleVariants({ variant: style.variant ?? variant, size: style.size ?? size }),
    'w-auto min-w-0 shrink-0 px-3 focus:z-10 focus-visible:z-10',
    'data-[spacing=0]:rounded-none data-[spacing=0]:shadow-none data-[spacing=0]:first:rounded-l-md data-[spacing=0]:last:rounded-r-md data-[spacing=0]:data-[variant=outline]:border-l-0 data-[spacing=0]:data-[variant=outline]:first:border-l',
    className,
  )}
  {...rest}
>
  {#snippet children()}
    {@render label?.()}
  {/snippet}
</ToggleGroupPrimitive.Item>
