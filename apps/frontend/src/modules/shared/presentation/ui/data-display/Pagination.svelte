<script lang="ts">
  import {
    Pagination as PaginationRoot,
    PaginationContent,
    PaginationEllipsis,
    PaginationItem,
    PaginationLink,
    PaginationNext,
    PaginationPrevious,
  } from '@shadcn';
  import type { PaginationInfo } from '@shared/domain/structs/pagination.struct.ts';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { generatePageNumbers } from './pagination.ts';
  import { paginationMessages } from './pagination.messages.ts';

  interface Props {
    paginationInfo: PaginationInfo;
    onPageChange: (page: number) => void;
    class?: string;
  }

  let { paginationInfo, onPageChange, class: className }: Props = $props();

  const pageItems = $derived(generatePageNumbers(paginationInfo.page, paginationInfo.totalPages));
  const start = $derived((paginationInfo.page - 1) * paginationInfo.pageSize + 1);
  const end = $derived(Math.min(paginationInfo.page * paginationInfo.pageSize, paginationInfo.totalCount));
</script>

<div class={cn('flex flex-col items-center gap-2', className)}>
  <PaginationRoot>
    <PaginationContent class="flex-wrap justify-center">
      <PaginationItem>
        <PaginationPrevious
          onclick={() => paginationInfo.hasPrevious && onPageChange(paginationInfo.page - 1)}
          class={paginationInfo.hasPrevious
            ? 'cursor-pointer'
            : 'pointer-events-none opacity-50'}
        />
      </PaginationItem>

      {#each pageItems as item, index (item === 'ellipsis' ? `ellipsis-${index}` : item)}
        <PaginationItem>
          {#if item === 'ellipsis'}
            <PaginationEllipsis />
          {:else}
            <PaginationLink
              isActive={item === paginationInfo.page}
              onclick={() => onPageChange(item)}
              class="cursor-pointer"
            >
              {item}
            </PaginationLink>
          {/if}
        </PaginationItem>
      {/each}

      <PaginationItem>
        <PaginationNext
          onclick={() => paginationInfo.hasNext && onPageChange(paginationInfo.page + 1)}
          class={paginationInfo.hasNext ? 'cursor-pointer' : 'pointer-events-none opacity-50'}
        />
      </PaginationItem>
    </PaginationContent>
  </PaginationRoot>

  <p class="text-sm text-muted-foreground">
    {t(paginationMessages.showing(start, end, paginationInfo.totalCount))}
  </p>
</div>
