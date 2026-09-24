<script lang="ts">
  import ChevronLeftIcon from '@lucide/svelte/icons/chevron-left';
  import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
  import { Button } from '@shadcn';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { secretMessages } from '../secret.messages.ts';

  interface Props {
    page: number;
    totalPages: number;
    totalItems: number;
    itemsPerPage: number;
    onPageChange: (page: number) => void;
  }

  let { page, totalPages, totalItems, itemsPerPage, onPageChange }: Props = $props();

  const firstItem = $derived(totalItems === 0 ? 0 : (page - 1) * itemsPerPage + 1);
  const lastItem = $derived(Math.min(page * itemsPerPage, totalItems));
</script>

<!-- Not mounted. Hand-rolled, not the shared `Pagination`: see AGENTS.md. -->
<div
  class="flex items-center justify-between gap-4 rounded-xl border border-border/60 bg-card px-4 py-3"
>
  <p class="text-sm text-muted-foreground">
    {t(secretMessages.showingRange(firstItem, lastItem, totalItems))}
  </p>
  <div class="flex items-center gap-2">
    <Button
      variant="outline"
      size="sm"
      disabled={page <= 1}
      onclick={() => onPageChange(page - 1)}
      class="h-8 gap-1"
    >
      <ChevronLeftIcon class="size-4" />
      <span class="hidden sm:inline">{t(secretMessages.previousPage)}</span>
    </Button>
    <span class="min-w-fit text-sm text-muted-foreground">{page} / {totalPages}</span>
    <Button
      variant="outline"
      size="sm"
      disabled={page >= totalPages}
      onclick={() => onPageChange(page + 1)}
      class="h-8 gap-1"
    >
      <span class="hidden sm:inline">{t(secretMessages.nextPage)}</span>
      <ChevronRightIcon class="size-4" />
    </Button>
  </div>
</div>
