<script lang="ts">
  import { FeatureHeader } from '@shared/presentation/ui-svelte';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { createRolesPage } from '../roles-page.state.svelte.ts';
  import RoleDetailPanel from './components/RoleDetailPanel.svelte';
  import RoleListItem from './components/RoleListItem.svelte';
  import RoleFormDialog from './components/role-form/RoleFormDialog.svelte';
  import { rolesMessages } from './roles.messages.ts';

  const page = createRolesPage();
</script>

<!--
  Access-control administration: the role catalog on the left, one role's
  permissions and grants on the right.

  Holding `MANAGE_ROLES` is what opens this page *and* what carries grant
  management with it, so the whole screen is behind one gate — but the two
  destructive controls still ask for themselves, because the route guard is not
  the thing the backend enforces.
-->
<div class="flex h-full min-h-0 w-full flex-col gap-4">
  <FeatureHeader
    count={page.roles.length}
    label={t(rolesMessages.role)}
    pluralLabel={t(rolesMessages.roles)}
    newLabel={t(rolesMessages.createRole)}
    onNew={page.canManageRoles ? page.openCreate : undefined}
    {...page.selection.headerProps}
  />

  <div
    class="grid min-h-0 flex-1 grid-cols-1 gap-4 lg:grid-cols-[minmax(0,1.1fr)_minmax(0,1fr)]"
  >
    <div class="flex min-h-0 flex-col gap-2 overflow-y-auto pr-1">
      {#if page.roles.length === 0}
        <p
          class="rounded-xl border border-dashed border-slate-200 py-10 text-center text-sm text-muted-foreground"
        >
          {t(rolesMessages.noRoles)}
        </p>
      {:else}
        {#each page.roles as role (role.id)}
          <RoleListItem
            {role}
            memberCount={page.memberCountOf(role.id)}
            active={role.id === page.activeRoleId}
            selected={page.selection.isSelected(role.id)}
            selectable={page.isSelectable(role)}
            onOpen={() => page.open(role.id)}
            onToggleSelect={() => page.selection.select(role.id)}
          />
        {/each}
      {/if}
    </div>

    <div class="min-h-0 rounded-xl border bg-background p-5 shadow-sm">
      <RoleDetailPanel role={page.activeRole} onEdit={page.openEdit} />
    </div>
  </div>

  <RoleFormDialog open={page.formOpen} role={page.editingRole} onClose={page.closeForm} />
</div>
