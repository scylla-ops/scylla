<script lang="ts">
  import MoreHorizontalIcon from '@lucide/svelte/icons/more-horizontal';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { paginationMessages } from './pagination.messages.ts';

  interface Props {
    class?: string;
  }

  let { class: className }: Props = $props();
</script>

<span
  aria-hidden="true"
  data-slot="pagination-ellipsis"
  class={cn('flex size-9 items-center justify-center', className)}
>
  <MoreHorizontalIcon class="size-4" />
  <span class="sr-only">{t(paginationMessages.morePages)}</span>
</span>
