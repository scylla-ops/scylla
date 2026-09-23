<script lang="ts">
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { isUnseen } from '../whats-new.svelte.ts';
  import { layoutMessages } from './layout.messages.ts';

  let { highlightId }: { highlightId: string } = $props();
</script>

{#if isUnseen(highlightId)}
  <span class="relative ml-auto inline-flex group-data-[collapsible=icon]:hidden">
    <span class="absolute inset-0 animate-ping rounded-full bg-primary opacity-40"></span>
    <span
      class="relative rounded-full bg-primary px-1.5 py-0.5 text-[10px] font-bold leading-none text-primary-foreground"
    >
      {t(layoutMessages.newBadge)}
    </span>
  </span>
{/if}
