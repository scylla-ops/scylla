<script lang="ts">
  import { Badge } from '@shadcn-svelte';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { RoleEntity } from '../../../../domain/entities/role.entity.ts';
  import { permissionLabelOf } from '../../../utils/permission-mapping.ts';
  import { rolesMessages } from '../../roles.messages.ts';

  interface Props {
    role: RoleEntity;
  }

  let { role }: Props = $props();
</script>

<section class="flex flex-col gap-2">
  <h3 class="text-xs font-semibold tracking-wider text-muted-foreground uppercase">
    {t(rolesMessages.permissions)}
  </h3>
  {#if role.access.kind === 'fullControl'}
    <p class="text-sm text-muted-foreground">{t(rolesMessages.grantsFullControl)}</p>
  {:else if role.access.kind === 'restricted'}
    {#if role.access.permissions.length === 0}
      <p class="text-sm text-muted-foreground">{t(rolesMessages.noPermissions)}</p>
    {:else}
      <div class="flex flex-wrap gap-1.5">
        {#each role.access.permissions as permission (permission)}
          <!-- Labelled against the *role's* scope, so a project permission
               conferred by an organization role says "every project". -->
          <Badge variant="outline" class="font-normal">
            {permissionLabelOf(permission, role.scope)}
          </Badge>
        {/each}
      </div>
    {/if}
  {:else}
    <p class="text-sm text-muted-foreground">{t(rolesMessages.unknownAccess)}</p>
  {/if}
</section>
