<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';
  import { Dialog as SheetPrimitive } from 'bits-ui';
  import { cn } from '@shared/presentation/utils';
  import SheetContent from './sheet-content.svelte';
  import { getSidebar } from './sidebar-state.svelte.ts';

  type Props = HTMLAttributes<HTMLDivElement> & {
    side?: 'left' | 'right';
    variant?: 'sidebar' | 'floating' | 'inset';
    collapsible?: 'offcanvas' | 'icon' | 'none';
    children?: Snippet;
  };

  let {
    side = 'left',
    variant = 'sidebar',
    collapsible = 'offcanvas',
    class: className,
    children,
    ...rest
  }: Props = $props();

  const sidebar = getSidebar();
</script>

{#if collapsible === 'none'}
  <div
    data-slot="sidebar"
    class={cn(
      'bg-sidebar text-sidebar-foreground flex h-full w-(--sidebar-width) flex-col',
      className,
    )}
    {...rest}
  >
    {@render children?.()}
  </div>
{:else if sidebar.isMobile}
  <SheetPrimitive.Root bind:open={sidebar.openMobile}>
    <SheetContent
      data-sidebar="sidebar"
      data-slot="sidebar"
      data-mobile="true"
      class="bg-sidebar text-sidebar-foreground w-(--sidebar-width) p-0 [&>button]:hidden"
      style="--sidebar-width: 18rem;"
      {side}
    >
      <SheetPrimitive.Title class="sr-only">Sidebar</SheetPrimitive.Title>
      <SheetPrimitive.Description class="sr-only">Displays the mobile sidebar.</SheetPrimitive.Description>
      <div class="flex h-full w-full flex-col">
        {@render children?.()}
      </div>
    </SheetContent>
  </SheetPrimitive.Root>
{:else}
  <div
    class="group peer text-sidebar-foreground hidden md:block"
    data-state={sidebar.state}
    data-collapsible={sidebar.state === 'collapsed' ? collapsible : ''}
    data-variant={variant}
    data-side={side}
    data-slot="sidebar"
  >
    <div
      data-slot="sidebar-gap"
      class={cn(
        'relative w-(--sidebar-width) bg-transparent transition-[width] duration-200 ease-linear',
        'group-data-[collapsible=offcanvas]:w-0',
        'group-data-[side=right]:rotate-180',
        variant === 'floating' || variant === 'inset'
          ? 'group-data-[collapsible=icon]:w-[calc(var(--sidebar-width-icon)+(--spacing(4)))]'
          : 'group-data-[collapsible=icon]:w-(--sidebar-width-icon)',
      )}
    ></div>
    <div
      data-slot="sidebar-container"
      class={cn(
        'fixed inset-y-0 z-10 hidden h-svh w-(--sidebar-width) transition-[left,right,width] duration-200 ease-linear md:flex',
        side === 'left'
          ? 'left-0 group-data-[collapsible=offcanvas]:left-[calc(var(--sidebar-width)*-1)]'
          : 'right-0 group-data-[collapsible=offcanvas]:right-[calc(var(--sidebar-width)*-1)]',
        variant === 'floating' || variant === 'inset'
          ? 'p-2 group-data-[collapsible=icon]:w-[calc(var(--sidebar-width-icon)+(--spacing(4))+2px)]'
          : 'group-data-[collapsible=icon]:w-(--sidebar-width-icon) group-data-[side=left]:border-r group-data-[side=right]:border-l',
        className,
      )}
      {...rest}
    >
      <div
        data-sidebar="sidebar"
        data-slot="sidebar-inner"
        class="bg-sidebar group-data-[variant=floating]:border-sidebar-border flex h-full w-full flex-col group-data-[variant=floating]:rounded-lg group-data-[variant=floating]:border group-data-[variant=floating]:shadow-sm"
      >
        {@render children?.()}
      </div>
    </div>
  </div>
{/if}
