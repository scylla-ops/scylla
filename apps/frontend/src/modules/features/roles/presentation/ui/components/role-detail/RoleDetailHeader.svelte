<script lang="ts">
  import PencilIcon from '@lucide/svelte/icons/pencil';
  import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';
  import { can, Permission } from '@platform/authz';
  import { Badge } from '@shadcn-svelte';
  import { GatedButton } from '@shared/presentation/ui-svelte';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { RoleEntity } from '../../../../domain/entities/role.entity.ts';
  import { scopeLabelOf } from '../../../utils/permission-mapping.ts';
  import { rolesMessages } from '../../roles.messages.ts';

  interface Props {
    role: RoleEntity;
    onEdit: (role: RoleEntity) => void;
  }

  let { role, onEdit }: Props = $props();

  const originLabel = $derived(
    {
      builtin: t(rolesMessages.builtin),
      custom: t(rolesMessages.custom),
      // A role whose origin the backend did not name — an older build, or a
      // kind added since. "Unknown" is feminine here: it agrees with "origine".
      unknown: t(rolesMessages.unknownOrigin),
    }[role.origin.kind],
  );

  const canEdit = $derived(can(Permission.MANAGE_ROLES));
</script>

<section class="flex flex-col gap-4">
  <div class="flex items-start justify-between gap-4">
    <div class="flex min-w-0 items-start gap-3">
      <div class="flex size-11 shrink-0 items-center justify-center rounded-xl bg-primary/10">
        <ShieldCheckIcon class="size-5 text-primary" />
      </div>
      <div class="min-w-0">
        <h2 class="truncate text-xl font-bold tracking-tight">{role.name}</h2>
        <p class="text-sm text-muted-foreground">
          {role.description || t(rolesMessages.noDescription)}
        </p>
      </div>
    </div>
    <GatedButton
      allowed={canEdit}
      deniedReason={t(rolesMessages.editDenied)}
      variant="outline"
      disabled={role.origin.kind === 'builtin'}
      onclick={() => onEdit(role)}
    >
      <PencilIcon class="size-4" />
      {t(rolesMessages.edit)}
    </GatedButton>
  </div>
  <div class="flex flex-wrap gap-2">
    <Badge variant="secondary">{scopeLabelOf(role.scope)}</Badge>
    <Badge variant="outline">{originLabel}</Badge>
    {#if role.access.kind === 'fullControl'}
      <Badge>{t(rolesMessages.fullControl)}</Badge>
    {/if}
  </div>
</section>
