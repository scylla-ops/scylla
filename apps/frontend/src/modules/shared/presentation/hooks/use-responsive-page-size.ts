import { useCallback, useEffect, useState } from 'react';

const MIN_PAGE_SIZE = 5;
const MAX_PAGE_SIZE = 50;
const DEFAULT_ROW_HEIGHT = 61;
const DEFAULT_HEADER_HEIGHT = 44;

interface UseResponsivePageSizeOptions {
  rowHeight?: number;
  headerHeight?: number;
}

const computePageSize = (containerHeight: number, rowHeight: number, headerHeight: number): number => {
  const rows = Math.floor((containerHeight - headerHeight) / rowHeight);
  return Math.min(MAX_PAGE_SIZE, Math.max(MIN_PAGE_SIZE, rows));
};

export const useResponsivePageSize = (options?: UseResponsivePageSizeOptions) => {
  const rowHeight = options?.rowHeight ?? DEFAULT_ROW_HEIGHT;
  const headerHeight = options?.headerHeight ?? DEFAULT_HEADER_HEIGHT;
  const [container, setContainer] = useState<HTMLElement | null>(null);
  const [pageSize, setPageSize] = useState(MIN_PAGE_SIZE);

  useEffect(() => {
    if (!container) return;

    const observer = new ResizeObserver(([entry]) => {
      setPageSize(computePageSize(entry.contentRect.height, rowHeight, headerHeight));
    });
    observer.observe(container);
    return () => observer.disconnect();
  }, [container, rowHeight, headerHeight]);

  const containerRef = useCallback(
    (node: HTMLElement | null) => {
      setContainer(node);
      // Read the size synchronously on attach, in the same commit the
      // container replaces the loading placeholder - waiting for the
      // observer's own (async) first callback would render once at
      // MIN_PAGE_SIZE and visibly crop down a moment later.
      if (node) setPageSize(computePageSize(node.clientHeight, rowHeight, headerHeight));
    },
    [rowHeight, headerHeight],
  );

  return { pageSize, containerRef };
};
