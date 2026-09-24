<script lang="ts">
  import { authorizationReady, can } from '@platform/authz';
  import { contextStore } from '@platform/context';
  import type { NavEntry } from '@platform/routing';
  import {
    Sidebar,
    SidebarContent,
    SidebarFooter,
    SidebarGroup,
    SidebarRail,
    Skeleton,
  } from '@shadcn';
  import { toRune } from '@shared/presentation/stores/to-rune.svelte.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { slugifyOrgName } from '@shared/utils/slug.ts';
  import { navSectionsFor } from '../nav-sections.ts';
  import { highlightIdForNav } from '../whats-new.svelte.ts';
  import OrganizationSelector from './context-selector/OrganizationSelector.svelte';
  import LanguageSelector from './LanguageSelector.svelte';
  import { layoutMessages } from './layout.messages.ts';
  import { NAV_SECTION_CARD_CLASS, NAV_SECTION_DIVIDER_CLASS } from './nav-classes.ts';
  import NavMain from './NavMain.svelte';
  import NavUser from './NavUser.svelte';

  let { navEntries }: { navEntries: readonly NavEntry[] } = $props();

  const context = toRune(contextStore);
  const organizationName = $derived(context().organization.name);
  const ready = $derived(authorizationReady());

  const sections = $derived(
    navSectionsFor({
      entries: navEntries,
      prefix: organizationName ? `/${slugifyOrgName(organizationName)}` : '',
      can: permission => can(permission),
      translate: t,
      highlightIdFor: highlightIdForNav,
      titles: {
        organization: t(layoutMessages.organization),
        system: t(layoutMessages.system),
      },
      organizationHeader,
    }),
  );
</script>

{#snippet organizationHeader()}
  <OrganizationSelector />
{/snippet}

<Sidebar variant="inset" collapsible="icon">
  <SidebarContent class="gap-3 px-2 py-2 group-data-[collapsible=icon]:px-0">
    {#if ready}
      <NavMain {sections} />
    {:else}
      <SidebarGroup class={NAV_SECTION_CARD_CLASS}>
        <OrganizationSelector />
        <div class={NAV_SECTION_DIVIDER_CLASS}></div>
        <div class="flex flex-col gap-1">
          {#each [0, 1, 2, 3] as index (index)}
            <div
              class="flex h-8 items-center gap-2 px-2 group-data-[collapsible=icon]:size-8 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:px-0"
            >
              <Skeleton class="size-4 shrink-0 rounded" />
              <Skeleton class="h-4 flex-1 group-data-[collapsible=icon]:hidden" />
            </div>
          {/each}
        </div>
      </SidebarGroup>
    {/if}
  </SidebarContent>
  <SidebarFooter class="flex flex-col gap-2">
    <LanguageSelector />
    <NavUser />
  </SidebarFooter>
  <SidebarRail label={t(layoutMessages.toggleSidebar)} />
</Sidebar>
