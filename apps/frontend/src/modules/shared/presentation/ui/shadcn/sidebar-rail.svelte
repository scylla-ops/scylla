<script lang="ts">
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import { cn } from '@shared/presentation/utils';
  import { getSidebar } from './sidebar-state.svelte.ts';

  type Props = HTMLButtonAttributes & { label: string };

  let { class: className, label, ...rest }: Props = $props();

  const sidebar = getSidebar();
</script>

<button
  data-sidebar="rail"
  data-slot="sidebar-rail"
  aria-label={label}
  tabindex={-1}
  onclick={() => sidebar.toggle()}
  title={label}
  class={cn(
    'hover:after:bg-sidebar-border absolute inset-y-0 z-20 hidden w-4 -translate-x-1/2 transition-all ease-linear group-data-[side=left]:-right-4 group-data-[side=right]:left-0 after:absolute after:inset-y-0 after:left-1/2 after:w-[2px] sm:flex',
    'in-data-[side=left]:cursor-w-resize in-data-[side=right]:cursor-e-resize',
    '[[data-side=left][data-state=collapsed]_&]:cursor-e-resize [[data-side=right][data-state=collapsed]_&]:cursor-w-resize',
    'hover:group-data-[collapsible=offcanvas]:bg-sidebar group-data-[collapsible=offcanvas]:translate-x-0 group-data-[collapsible=offcanvas]:after:left-full',
    '[[data-side=left][data-collapsible=offcanvas]_&]:-right-2',
    '[[data-side=right][data-collapsible=offcanvas]_&]:-left-2',
    className,
  )}
  {...rest}
></button>
