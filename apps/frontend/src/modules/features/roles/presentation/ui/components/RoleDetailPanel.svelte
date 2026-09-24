<script lang="ts">
  import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { RoleEntity } from '../../../domain/entities/role.entity.ts';
  import { rolesMessages } from '../roles.messages.ts';
  import RoleDetailGrantList from './role-detail/RoleDetailGrantList.svelte';
  import RoleDetailHeader from './role-detail/RoleDetailHeader.svelte';
  import RoleDetailPermissions from './role-detail/RoleDetailPermissions.svelte';

  interface Props {
    role: RoleEntity | null;
    onEdit: (role: RoleEntity) => void;
  }

  let { role, onEdit }: Props = $props();
</script>

{#if !role}
  <div
    class="flex h-full flex-col items-center justify-center gap-3 text-center text-muted-foreground"
  >
    <div class="flex size-12 items-center justify-center rounded-xl border">
      <ShieldCheckIcon class="size-6 text-slate-400" />
    </div>
    <p class="text-sm">{t(rolesMessages.selectARole)}</p>
  </div>
{:else}
  <!-- Keyed on the role: its ViewModels are built from it. -->
  {#key role.id}
    <div class="flex h-full flex-col gap-6 overflow-y-auto p-4">
      <RoleDetailHeader {role} {onEdit} />
      <RoleDetailPermissions {role} />
      <RoleDetailGrantList {role} />
    </div>
  {/key}
{/if}
