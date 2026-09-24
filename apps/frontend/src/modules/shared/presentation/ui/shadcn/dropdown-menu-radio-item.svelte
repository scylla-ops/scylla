<script lang="ts">
  import type { Snippet } from 'svelte';
  import { DropdownMenu as DropdownMenuPrimitive } from 'bits-ui';
  import CircleIcon from '@lucide/svelte/icons/circle';
  import { cn } from '@shared/presentation/utils';

  type Props = Omit<DropdownMenuPrimitive.RadioItemProps, 'children'> & { children?: Snippet };

  let { class: className, children: label, ...rest }: Props = $props();
</script>

<DropdownMenuPrimitive.RadioItem
  data-slot="dropdown-menu-radio-item"
  class={cn(
    "focus:bg-accent focus:text-accent-foreground data-highlighted:bg-accent data-highlighted:text-accent-foreground relative flex cursor-default items-center gap-2 rounded-sm py-1.5 pr-2 pl-8 text-sm outline-hidden select-none data-[disabled]:pointer-events-none data-[disabled]:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
    className,
  )}
  {...rest}
>
  {#snippet children({ checked })}
    <span class="pointer-events-none absolute left-2 flex size-3.5 items-center justify-center">
      {#if checked}
        <CircleIcon class="size-2 fill-current" />
      {/if}
    </span>
    {@render label?.()}
  {/snippet}
</DropdownMenuPrimitive.RadioItem>
