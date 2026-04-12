import { useState, useCallback, useMemo } from 'react';
import { DEFAULT_PAGE_SIZE } from '@/modules/shared/domain/types/Pagination.ts';
import type { PaginationParams } from '@/modules/shared/domain/types/Pagination.ts';

interface UsePaginationOptions {
  initialPage?: number;
  initialPageSize?: number;
}

export const usePagination = (options?: UsePaginationOptions) => {
  const [page, setPageState] = useState(options?.initialPage ?? 1);
  const [pageSize, setPageSizeState] = useState(options?.initialPageSize ?? DEFAULT_PAGE_SIZE);

  const setPage = useCallback((newPage: number) => {
    setPageState(Math.max(1, newPage));
  }, []);

  const setPageSize = useCallback((newSize: number) => {
    setPageSizeState(Math.min(100, Math.max(1, newSize)));
    setPageState(1);
  }, []);

  const paginationParams = useMemo<PaginationParams>(
    () => ({ page, pageSize }),
    [page, pageSize],
  );

  return { page, pageSize, setPage, setPageSize, paginationParams };
};
