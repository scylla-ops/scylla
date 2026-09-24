<script lang="ts">
  import ChevronLeftIcon from '@lucide/svelte/icons/chevron-left';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import PaginationLink from './pagination-link.svelte';
  import { paginationMessages } from './pagination.messages.ts';

  interface Props {
    class?: string;
    onclick?: (event: MouseEvent) => void;
  }

  let { class: className, onclick }: Props = $props();
</script>

<PaginationLink
  aria-label={t(paginationMessages.goToPrevious)}
  size="default"
  class={cn('gap-1 px-2.5 sm:pl-2.5', className)}
  {onclick}
>
  <ChevronLeftIcon />
  <span class="hidden sm:block">{t(paginationMessages.previous)}</span>
</PaginationLink>
