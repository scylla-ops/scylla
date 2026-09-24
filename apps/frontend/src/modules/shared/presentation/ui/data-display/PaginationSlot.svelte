<script lang="ts">
  import type { PaginationInfo } from '@shared/domain/structs/pagination.struct.ts';
  import Pagination from './Pagination.svelte';

  interface Props {
    paginationInfo: PaginationInfo | undefined;
    onPageChange: (page: number) => void;
  }

  let { paginationInfo, onPageChange }: Props = $props();

  /** Rendered, then hidden: the slot keeps its height when it is empty. */
  const RESERVED_SLOT: PaginationInfo = {
    totalCount: 0,
    page: 1,
    pageSize: 0,
    totalPages: 1,
    hasNext: false,
    hasPrevious: false,
  };

  const isVisible = $derived(paginationInfo !== undefined && paginationInfo.totalPages > 1);
</script>

<!-- Always takes the bar's height: the area above is measured to decide how many rows fit. -->
<div class="shrink-0 pt-2" class:invisible={!isVisible}>
  <Pagination paginationInfo={paginationInfo ?? RESERVED_SLOT} {onPageChange} />
</div>
