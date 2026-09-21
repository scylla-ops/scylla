<script lang="ts">
  import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
  } from '@shadcn-svelte';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { RoleEntity } from '../../../../domain/entities/role.entity.ts';
  import { rolesMessages } from '../../roles.messages.ts';
  import RoleForm from './RoleForm.svelte';

  interface Props {
    open: boolean;
    /** The role being edited, or `null` when creating a new one. */
    role: RoleEntity | null;
    onClose: () => void;
  }

  let { open, role, onClose }: Props = $props();
</script>

<Dialog
  {open}
  onOpenChange={next => {
    if (!next) onClose();
  }}
>
  <DialogContent class="flex max-h-[85vh] max-w-xl flex-col">
    <DialogHeader class="space-y-3">
      <DialogTitle class="flex items-center gap-2.5 text-lg font-semibold">
        <div class="flex size-8 items-center justify-center rounded-lg bg-primary/10">
          <ShieldCheckIcon class="size-4 text-primary" />
        </div>
        <span>{role ? t(rolesMessages.editRoleTitle) : t(rolesMessages.createRole)}</span>
      </DialogTitle>
      <DialogDescription>{t(rolesMessages.roleFormSubtitle)}</DialogDescription>
    </DialogHeader>

    <!-- `{#key}` is the reset: a new opening builds a new form, seeded from the
         role it was opened for. -->
    {#key open}
      <RoleForm {role} onDone={onClose} />
    {/key}
  </DialogContent>
</Dialog>
