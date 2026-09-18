import type { Action } from 'svelte/action';
import {
  DEFAULT_PAGE_SIZE,
  type PaginationInfo,
  type PaginationParams,
} from '@shared/domain/structs/pagination.struct.ts';
import { createMeasuredHeight } from './measured-height.svelte.ts';
import { MIN_PAGE_SIZE, computePageSize } from './responsive-page-size.ts';

export interface PaginationOptions {
  initialPage?: number;
  initialPageSize?: number;
  /** Size the page to the container `measure` is attached to. */
  responsive?: boolean;
  rowHeight?: number;
  headerHeight?: number;
}

export interface Pagination {
  readonly page: number;
  readonly pageSize: number;
  readonly paginationParams: PaginationParams;
  readonly paginationInfo: PaginationInfo | undefined;
  /** False only while a responsive container has yet to be measured. */
  readonly isPageSizeReady: boolean;
  setPage: (page: number) => void;
  setPageSize: (size: number) => void;
  updatePaginationInfo: (info: PaginationInfo | undefined) => void;
  /** `use:measure` on the list container, when `responsive` is on. */
  measure: Action<HTMLElement>;
}

/**
 * Local page state merged with what the server reports — the Svelte
 * counterpart of `use-pagination.ts`.
 *
 * One thing is deliberately different, and it is the point of the port: React
 * held `pageSize` in state and ran an effect to copy the measured size into it.
 * Here the size is *derived* from the measurement, so there is no effect, no
 * mirror state, and no frame where the two disagree. `pageSizeIsFixed` is the
 * only thing that has to be remembered, because "the user picked a size" is a
 * fact about the past that nothing else records.
 */
export const createPagination = (options?: PaginationOptions): Pagination => {
  const isResponsive = (options?.responsive ?? false) && options?.initialPageSize === undefined;
  const measured = createMeasuredHeight();

  let page = $state(Math.max(1, options?.initialPage ?? 1));
  let chosenPageSize = $state(options?.initialPageSize);
  let serverInfo = $state<PaginationInfo | undefined>(undefined);

  const measuredPageSize = $derived(
    measured.height === null
      ? null
      : computePageSize(measured.height, options?.rowHeight, options?.headerHeight),
  );

  const pageSize = $derived.by(() => {
    if (chosenPageSize !== undefined) return chosenPageSize;
    if (!isResponsive) return DEFAULT_PAGE_SIZE;
    return measuredPageSize ?? MIN_PAGE_SIZE;
  });

  return {
    get page() {
      return page;
    },
    get pageSize() {
      return pageSize;
    },
    get paginationParams() {
      return { page, pageSize };
    },
    get paginationInfo() {
      if (!serverInfo) return undefined;
      return {
        totalCount: serverInfo.totalCount,
        totalPages: serverInfo.totalPages,
        page,
        pageSize,
        hasNext: page < serverInfo.totalPages,
        hasPrevious: page > 1,
      };
    },
    get isPageSizeReady() {
      return !isResponsive || measuredPageSize !== null;
    },

    setPage: (newPage: number) => {
      page = Math.max(1, newPage);
    },

    setPageSize: (newSize: number) => {
      chosenPageSize = Math.min(100, Math.max(1, newSize));
      page = 1;
    },

    updatePaginationInfo: (info: PaginationInfo | undefined) => {
      serverInfo = info;
      // The page the user is on can stop existing — deleting the last item on
      // the last page. Fall back to the last one that does.
      if (info && info.totalPages > 0 && page > info.totalPages) page = info.totalPages;
    },

    measure: measured.measure,
  };
};
