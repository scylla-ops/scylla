<script lang="ts">
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { errorStateMessages } from './error-state.messages.ts';

  interface Props {
    title?: string;
    message?: string;
  }

  let { title, message }: Props = $props();
</script>

<div class="flex h-full items-center justify-center">
  <div class="space-y-2 text-center">
    <p class="text-lg font-semibold text-destructive">{title ?? t(errorStateMessages.title)}</p>
    <p class="text-sm text-muted-foreground">{message ?? t(errorStateMessages.message)}</p>
  </div>
</div>
