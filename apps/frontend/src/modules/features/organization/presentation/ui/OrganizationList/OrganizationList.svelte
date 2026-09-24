<script lang="ts">
  import type { Component, Snippet } from 'svelte';
  import Building2Icon from '@lucide/svelte/icons/building-2';
  import PencilIcon from '@lucide/svelte/icons/pencil';
  import TrashIcon from '@lucide/svelte/icons/trash';
  import UsersIcon from '@lucide/svelte/icons/users';
  import { can, Permission } from '@platform/authz';
  import { navigateTo, contextStore } from '@platform/context';
  import { createMutation, createQuery } from '@platform/query';
  import { Skeleton } from '@shadcn';
  import {
    ConfirmOperationAlertDialog,
    ContextItem,
    IconButton,
  } from '@shared/presentation/ui';
  import { toRune } from '@shared/presentation/stores/to-rune.svelte.ts';
  import { slugifyOrgName } from '@shared/utils/slug.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { organizationMutations, organizationQueries } from '../../organization.queries.ts';
  import { organizationMessages } from '../organization.messages.ts';
  import EditOrganizationDialog from '../EditOrganizationDialog/EditOrganizationDialog.svelte';
  import OrganizationRow from '../OrganizationRow.svelte';

  type RowProps = {
    class?: string;
    onSelect?: () => void;
    children?: Snippet;
  };

  let { row: Row = OrganizationRow }: { row?: Component<RowProps> } = $props();

  const organizationsQuery = createQuery(() => organizationQueries.mine());
  const organizations = $derived(organizationsQuery.data);

  const deleteOrganization = createMutation(() => organizationMutations.remove());

  const context = toRune(contextStore);
  const currentOrganizationId = $derived(context().organization.id);

  let editOrg = $state<{ id: string; name: string; description?: string } | null>(null);
  let deleteOrgId = $state<string | null>(null);

  const selectOrganization = (id: string, name: string) => {
    contextStore.getState().setOrganization(id, name);
    navigateTo(`/${slugifyOrgName(name)}/dashboard`);
  };

  /**
   * Deleting the organization you are looking at leaves every scoped URL
   * pointing at nothing, so the context moves to another one first — and to
   * none at all when it was the last.
   */
  const onDeleteOrganization = async () => {
    if (!deleteOrgId) return;

    const deletedId = deleteOrgId;
    await deleteOrganization.mutateAsync(deletedId);
    deleteOrgId = null;

    if (deletedId !== currentOrganizationId) return;

    const other = organizations?.find(organization => organization.id !== deletedId);
    contextStore.getState().setOrganization(other?.id ?? null, other?.name ?? null);
    if (other) navigateTo(`/${slugifyOrgName(other.name)}/dashboard`);
  };
</script>

{#if !organizations}
  {#each Array.from({ length: 3 }) as _, index (index)}
    <Row class="group">
      <div class="flex items-center gap-3 px-1 py-1">
        <Skeleton class="h-8 w-8 rounded-md" />
        <Skeleton class="h-4 w-24" />
      </div>
    </Row>
  {/each}
{:else}
  {#each organizations as organization (organization.id)}
    <Row
      class="group rounded-md transition-colors hover:bg-accent/70"
      onSelect={() => selectOrganization(organization.id, organization.name)}
    >
      <div class="flex w-full items-center">
        <div class="min-w-0 flex-1">
          <ContextItem
            name={organization.name}
            description={organization.description}
            icon={Building2Icon}
          />
        </div>
        <div class="flex gap-0.5 opacity-0 transition-opacity group-hover:opacity-100">
          {#if can(Permission.LIST_ORGANIZATION_MEMBERS, { organizationId: organization.id })}
            <IconButton
              icon={UsersIcon}
              tooltip={t(organizationMessages.members)}
              onclick={event => {
                event.stopPropagation();
                // The members page reads the organization from the context
                // store, so looking at another org's members means moving to
                // it — the row's own click does the same thing.
                contextStore.getState().setOrganization(organization.id, organization.name);
                navigateTo(`/${slugifyOrgName(organization.name)}/members`);
              }}
              class="h-7 w-7"
              iconClass="h-3.5 w-3.5"
            />
          {/if}
          {#if can(Permission.UPDATE_ORGANIZATION, { organizationId: organization.id })}
            <IconButton
              icon={PencilIcon}
              tooltip={t(organizationMessages.edit)}
              onclick={event => {
                event.stopPropagation();
                editOrg = {
                  id: organization.id,
                  name: organization.name,
                  description: organization.description,
                };
              }}
              class="h-7 w-7"
              iconClass="h-3.5 w-3.5"
            />
          {/if}
          {#if can(Permission.DELETE_ORGANIZATION, { organizationId: organization.id })}
            <IconButton
              icon={TrashIcon}
              tooltip={t(organizationMessages.delete)}
              onclick={event => {
                event.stopPropagation();
                deleteOrgId = organization.id;
              }}
              class="h-7 w-7 hover:bg-destructive-subtle hover:text-destructive"
              iconClass="h-3.5 w-3.5"
            />
          {/if}
        </div>
      </div>
    </Row>
  {/each}
{/if}

{#if editOrg}
  <EditOrganizationDialog
    open={!!editOrg}
    setOpen={open => {
      if (!open) editOrg = null;
    }}
    organization={editOrg}
  />
{/if}

<ConfirmOperationAlertDialog
  open={!!deleteOrgId}
  onOpenChange={open => {
    if (!open) deleteOrgId = null;
  }}
  onContinue={onDeleteOrganization}
/>
