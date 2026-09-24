<script lang="ts">
  import ShieldOffIcon from '@lucide/svelte/icons/shield-off';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { permissionDeniedMessages } from '../permission-denied.messages.ts';

  let { message }: { message?: string } = $props();
</script>

<div
  class="flex h-full w-full flex-col items-center justify-center gap-3 py-16 text-center text-muted-foreground"
>
  <div class="flex size-12 items-center justify-center rounded-xl bg-slate-100">
    <ShieldOffIcon class="size-6 text-slate-400" />
  </div>
  <p class="text-sm font-medium text-foreground">{t(permissionDeniedMessages.title)}</p>
  <p class="max-w-sm text-sm">{message ?? t(permissionDeniedMessages.askAdministrator)}</p>
</div>
