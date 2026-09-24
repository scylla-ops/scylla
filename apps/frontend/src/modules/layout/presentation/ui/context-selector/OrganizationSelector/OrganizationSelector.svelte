<script lang="ts">
  import Building2Icon from '@lucide/svelte/icons/building-2';
  import PlusIcon from '@lucide/svelte/icons/plus';
  import { can, Permission } from '@platform/authz';
  import { contextStore } from '@platform/context';
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
    getSidebar,
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
  } from '@shadcn';
  import { toRune } from '@shared/presentation/stores/to-rune.svelte.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { loadAddOrganizationDialog, loadOrganizationList } from '@/modules/features/organization';
  import { layoutMessages } from '../../layout.messages.ts';
  import CurrentContextDisplay from '../CurrentContextDisplay.svelte';

  const sidebar = getSidebar();
  const context = toRune(contextStore);
  const organizationName = $derived(context().organization.name);
  const canCreate = $derived(can(Permission.CREATE_ORGANIZATION));
  const label = $derived(t(layoutMessages.organization));

  let addOpen = $state(false);
</script>

<SidebarMenu>
  <SidebarMenuItem>
    <DropdownMenu>
      <DropdownMenuTrigger>
        {#snippet child({ props })}
          <SidebarMenuButton
            {...props}
            size="lg"
            class="rounded-lg px-2 transition-colors duration-200 hover:bg-accent focus:outline-none focus:ring-0 focus-visible:ring-0 data-[state=open]:bg-accent"
          >
            <CurrentContextDisplay
              name={organizationName || t(layoutMessages.selectOrganization)}
              description={label}
              icon={Building2Icon}
            />
          </SidebarMenuButton>
        {/snippet}
      </DropdownMenuTrigger>

      <DropdownMenuContent
        class="w-(--bits-dropdown-menu-anchor-width) min-w-56 rounded-xl border-border shadow-lg"
        align="start"
        side={sidebar.isMobile ? 'bottom' : 'right'}
        sideOffset={4}
      >
        <DropdownMenuLabel
          class="px-3 py-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground"
        >
          {label}
        </DropdownMenuLabel>

        {#await loadOrganizationList() then { default: OrganizationList }}
          <OrganizationList row={DropdownMenuItem} />
        {/await}

        {#if canCreate}
          <DropdownMenuSeparator class="bg-border" />
          <DropdownMenuItem
            class="group mx-1 mb-1 cursor-pointer gap-3 rounded-lg p-2 hover:bg-accent"
            onSelect={() => (addOpen = true)}
          >
            <div
              class="flex size-8 items-center justify-center rounded-md border border-border bg-background transition-colors group-hover:border-primary"
            >
              <PlusIcon
                class="size-4 text-muted-foreground transition-colors group-hover:text-primary"
              />
            </div>
            <div class="font-medium text-foreground group-hover:text-primary">
              Create new {label.toLowerCase()}
            </div>
          </DropdownMenuItem>
        {/if}
      </DropdownMenuContent>
    </DropdownMenu>
  </SidebarMenuItem>
</SidebarMenu>

{#await loadAddOrganizationDialog() then { default: AddOrganizationDialog }}
  <AddOrganizationDialog open={addOpen} setOpen={open => (addOpen = open)} />
{/await}
