<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import { mergeProps } from 'bits-ui';
  import { cn } from '@shared/presentation/utils';
  import { getSidebar } from './sidebar-state.svelte.ts';
  import {
    sidebarMenuButtonVariants,
    type SidebarMenuButtonSize,
    type SidebarMenuButtonVariant,
  } from './sidebar-variants.ts';
  import Tooltip from './tooltip.svelte';
  import TooltipContent from './tooltip-content.svelte';
  import TooltipTrigger from './tooltip-trigger.svelte';

  type Props = HTMLButtonAttributes & {
    isActive?: boolean;
    variant?: SidebarMenuButtonVariant;
    size?: SidebarMenuButtonSize;
    tooltip?: string;
    children?: Snippet;
  };

  let {
    isActive = false,
    variant = 'default',
    size = 'default',
    tooltip,
    class: className,
    children,
    ...rest
  }: Props = $props();

  const sidebar = getSidebar();
</script>

{#snippet button(props: Record<string, unknown>)}
  <button
    data-slot="sidebar-menu-button"
    data-sidebar="menu-button"
    data-size={size}
    data-active={isActive}
    class={cn(sidebarMenuButtonVariants({ variant, size }), className)}
    {...mergeProps(rest, props)}
  >
    {@render children?.()}
  </button>
{/snippet}

{#if !tooltip}
  {@render button({})}
{:else}
  <Tooltip>
    <TooltipTrigger>
      {#snippet child({ props })}
        {@render button(props)}
      {/snippet}
    </TooltipTrigger>
    <TooltipContent
      side="right"
      align="center"
      hidden={sidebar.state !== 'collapsed' || sidebar.isMobile}
    >
      {tooltip}
    </TooltipContent>
  </Tooltip>
{/if}
