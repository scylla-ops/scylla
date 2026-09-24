<script lang="ts">
  import AppWindowIcon from '@lucide/svelte/icons/app-window';
  import Building2Icon from '@lucide/svelte/icons/building-2';
  import FolderGit2Icon from '@lucide/svelte/icons/folder-git-2';
  import GlobeIcon from '@lucide/svelte/icons/globe';
  import UserIcon from '@lucide/svelte/icons/user';
  import XIcon from '@lucide/svelte/icons/x';
  import { can, Permission, PermissionScope, PrincipalKind } from '@platform/authz';
  import { Badge } from '@shadcn';
  import { IconButton } from '@shared/presentation/ui';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { RoleEntity } from '../../../../domain/entities/role.entity.ts';
  import { createGrantTargetLabels } from '../../../grant-target-labels.svelte.ts';
  import { createRoleAssignees } from '../../../role-assignees.state.svelte.ts';
  import { rolesMessages } from '../../roles.messages.ts';
  import GrantCreator from './GrantCreator.svelte';

  interface Props {
    role: RoleEntity;
  }

  let { role }: Props = $props();

  const assignees = createRoleAssignees(() => role);
  const targets = createGrantTargetLabels(() => role.scope);

  const SCOPE_ICON = {
    [PermissionScope.SYSTEM]: GlobeIcon,
    [PermissionScope.ORGANIZATION]: Building2Icon,
    [PermissionScope.PROJECT]: FolderGit2Icon,
    [PermissionScope.UNSPECIFIED]: GlobeIcon,
  };

  const canRevoke = $derived(can(Permission.MANAGE_SYSTEM_GRANTS));
</script>

<section class="flex min-h-0 flex-col gap-2">
  <div class="flex items-center justify-between">
    <h3 class="text-xs font-semibold tracking-wider text-muted-foreground uppercase">
      {t(rolesMessages.grants)} ({assignees.assignees.length})
    </h3>

    <GrantCreator {role} />
  </div>

  {#if assignees.assignees.length === 0}
    <p class="rounded-lg border border-dashed py-6 text-center text-sm text-muted-foreground">
      {t(rolesMessages.noGrants)}
    </p>
  {:else}
    <!-- Not `ScrollArea`: its root does not clip under a `max-h`. -->
    <div class="max-h-72 overflow-y-auto">
      <ul class="flex flex-col gap-2 pr-2">
        {#each assignees.assignees as { grant, label } (grant.id)}
          {@const isUser = grant.principal.kind === PrincipalKind.USER}
          {@const target = targets.labelFor(grant.scopeId)}
          {@const ScopeIcon = SCOPE_ICON[role.scope] ?? GlobeIcon}
          <li class="flex items-center gap-3 rounded-lg border px-3 py-2">
            <div class="flex size-9 shrink-0 items-center justify-center rounded-lg bg-primary/10">
              {#if isUser}
                <UserIcon class="size-4 text-primary" />
              {:else}
                <AppWindowIcon class="size-4 text-primary" />
              {/if}
            </div>
            <div class="flex min-w-0 flex-col items-start gap-1">
              <p class="max-w-full truncate font-medium text-foreground">{label}</p>
              <Badge variant="secondary" class="max-w-full gap-1 font-normal">
                <ScopeIcon class="size-3 shrink-0" />
                <span class="truncate">
                  {target.organizationName ? `${target.organizationName} / ` : ''}{target.name}
                </span>
              </Badge>
            </div>
            <IconButton
              icon={XIcon}
              tooltip={canRevoke ? t(rolesMessages.remove) : t(rolesMessages.revokeDenied)}
              disabled={!canRevoke}
              class="ml-auto hover:text-destructive"
              onclick={() => assignees.remove(grant.id)}
            />
          </li>
        {/each}
      </ul>
    </div>
  {/if}
</section>
