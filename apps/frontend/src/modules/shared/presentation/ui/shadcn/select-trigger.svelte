<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Select as SelectPrimitive } from 'bits-ui';
  import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
  import { cn } from '@shared/presentation/utils';

  type Props = SelectPrimitive.TriggerProps & {
    size?: 'sm' | 'default';
    children?: Snippet;
  };

  let { class: className, size = 'default', children, ...rest }: Props = $props();
</script>

<!--
  `role="combobox"` is ours, not bits-ui's. It already sets every other piece of
  the pattern on this button — `aria-haspopup="listbox"`, `aria-expanded`,
  `aria-activedescendant` — but leaves the role off, so the control announces as
  a plain button and no test can find it by the role its behaviour advertises.
  Radix set it; this restores it.
-->
<SelectPrimitive.Trigger
  data-slot="select-trigger"
  data-size={size}
  role="combobox"
  class={cn(
    "border-input data-placeholder:text-muted-foreground [&_svg:not([class*='text-'])]:text-muted-foreground focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive dark:bg-input/30 dark:hover:bg-input/50 flex w-fit items-center justify-between gap-2 rounded-md border bg-transparent px-3 py-2 text-sm whitespace-nowrap shadow-xs transition-[color,box-shadow] outline-none focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50 data-[size=default]:h-9 data-[size=sm]:h-8 *:data-[slot=select-value]:line-clamp-1 *:data-[slot=select-value]:flex *:data-[slot=select-value]:items-center *:data-[slot=select-value]:gap-2 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
    className,
  )}
  {...rest}
>
  {@render children?.()}
  <!-- Radix wrapped this in `Select.Icon asChild`; bits-ui has no such part, and
       the icon needs nothing from the primitive beyond being inside the trigger. -->
  <ChevronDownIcon class="size-4 opacity-50" />
</SelectPrimitive.Trigger>
