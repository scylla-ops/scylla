import { useState, useCallback, useMemo, useEffect } from 'react';
import {
  DEFAULT_PAGE_SIZE,
  type PaginationInfo,
  type PaginationParams,
} from '@shared/domain/structs/pagination.struct.ts';
import { useResponsivePageSize } from '@shared/presentation/hooks/use-responsive-page-size.ts';

interface UsePaginationOptions {
  initialPage?: number;
  initialPageSize?: number;
  responsive?: boolean;
  rowHeight?: number;
  headerHeight?: number;
}

export const usePagination = (options?: UsePaginationOptions) => {
  const isResponsive = (options?.responsive ?? false) && options?.initialPageSize === undefined;
  const {
    pageSize: responsivePageSize,
    isMeasured,
    containerRef,
  } = useResponsivePageSize({
    rowHeight: options?.rowHeight,
    headerHeight: options?.headerHeight,
  });
  const defaultPageSize = isResponsive ? responsivePageSize : DEFAULT_PAGE_SIZE;
  const [pageSizeIsFixed, setPageSizeIsFixed] = useState(options?.initialPageSize !== undefined);

  const [page, setPageState] = useState(options?.initialPage ?? 1);
  const [pageSize, setPageSizeState] = useState(options?.initialPageSize ?? defaultPageSize);
  const [serverPaginationInfo, setServerPaginationInfo] = useState<PaginationInfo | undefined>(
    undefined,
  );

  useEffect(() => {
    if (!pageSizeIsFixed && isResponsive) setPageSizeState(responsivePageSize);
  }, [responsivePageSize, pageSizeIsFixed, isResponsive]);

  const setPage = useCallback((newPage: number) => {
    setPageState(Math.max(1, newPage));
  }, []);

  const setPageSize = useCallback((newSize: number) => {
    setPageSizeIsFixed(true);
    setPageSizeState(Math.min(100, Math.max(1, newSize)));
    setPageState(1);
  }, []);

  const paginationParams = useMemo<PaginationParams>(() => ({ page, pageSize }), [page, pageSize]);

  const updatePaginationInfo = useCallback(
    (info: PaginationInfo | undefined) => {
      setServerPaginationInfo(info);
      // If current page exceeds total pages (e.g. after deleting all items on last page),
      // go back to the last available page.
      if (info && info.totalPages > 0 && page > info.totalPages) {
        setPageState(info.totalPages);
      }
    },
    [page],
  );

  const paginationInfo = useMemo<PaginationInfo | undefined>(() => {
    if (!serverPaginationInfo) return undefined;
    return {
      totalCount: serverPaginationInfo.totalCount,
      totalPages: serverPaginationInfo.totalPages,
      page,
      pageSize,
      hasNext: page < serverPaginationInfo.totalPages,
      hasPrevious: page > 1,
    };
  }, [serverPaginationInfo, page, pageSize]);

  return {
    page,
    pageSize,
    setPage,
    setPageSize,
    paginationParams,
    paginationInfo,
    updatePaginationInfo,
    containerRef,
    isPageSizeReady: !isResponsive || isMeasured,
  };
};
