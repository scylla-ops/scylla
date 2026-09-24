<script lang="ts">
  import type { PaginationInfo } from '@shared/domain/structs/pagination.struct.ts';
  import Pagination from './Pagination.svelte';

  interface Props {
    paginationInfo: PaginationInfo | undefined;
    onPageChange: (page: number) => void;
  }

  let { paginationInfo, onPageChange }: Props = $props();

  /** Drawn, then hidden: the slot has to keep its height even with nothing in it. */
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

<!--
  Always takes the bar's height: the table area above it is measured to decide
  how many rows fit, so a slot that appeared and disappeared would change the
  measurement that decides whether it is needed at all.
-->
<div class="shrink-0 pt-2" class:invisible={!isVisible}>
  <Pagination paginationInfo={paginationInfo ?? RESERVED_SLOT} {onPageChange} />
</div>
