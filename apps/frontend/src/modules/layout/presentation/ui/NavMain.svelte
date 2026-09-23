<script lang="ts">
  import { navigateTo } from '@platform/context';
  import { routePathname } from '@platform/routing';
  import {
    SidebarGroup,
    SidebarGroupLabel,
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
  } from '@shadcn-svelte';
  import type { NavItem, NavSection } from '../structs/nav-section.struct.ts';
  import { markSeen } from '../whats-new.svelte.ts';
  import { NAV_SECTION_CARD_CLASS, NAV_SECTION_DIVIDER_CLASS } from './nav-classes.ts';
  import NewBadge from './NewBadge.svelte';

  let { sections }: { sections: NavSection[] } = $props();

  const pathname = $derived(routePathname());

  const open = (item: NavItem) => {
    if (item.highlightId) markSeen(item.highlightId);
    navigateTo(item.url);
  };
</script>

{#each sections as section (section.title)}
  <SidebarGroup class={NAV_SECTION_CARD_CLASS}>
    {#if section.header}
      {@render section.header()}
      <div class={NAV_SECTION_DIVIDER_CLASS}></div>
    {:else}
      <SidebarGroupLabel class="group-data-[collapsible=icon]:hidden">
        {section.title}
      </SidebarGroupLabel>
    {/if}
    <SidebarMenu>
      {#each section.items as item (item.url)}
        <SidebarMenuItem
          class="transition-all duration-200 hover:scale-105 group-data-[collapsible=icon]:hover:scale-100"
        >
          <SidebarMenuButton
            tooltip={item.title}
            isActive={pathname.startsWith(item.url)}
            onclick={() => open(item)}
          >
            {#if item.icon}
              <item.icon />
            {/if}
            <span>{item.title}</span>
            {#if item.highlightId}
              <NewBadge highlightId={item.highlightId} />
            {/if}
          </SidebarMenuButton>
        </SidebarMenuItem>
      {/each}
    </SidebarMenu>
  </SidebarGroup>
{/each}
