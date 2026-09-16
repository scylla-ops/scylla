import { useCallback, useEffect, useState } from 'react';

const MIN_PAGE_SIZE = 5;
const MAX_PAGE_SIZE = 50;
const DEFAULT_ROW_HEIGHT = 61;
const DEFAULT_HEADER_HEIGHT = 44;

interface UseResponsivePageSizeOptions {
  rowHeight?: number;
  headerHeight?: number;
}

const computePageSize = (
  containerHeight: number,
  rowHeight: number,
  headerHeight: number,
): number => {
  const rows = Math.floor((containerHeight - headerHeight) / rowHeight);
  return Math.min(MAX_PAGE_SIZE, Math.max(MIN_PAGE_SIZE, rows));
};

export const useResponsivePageSize = (options?: UseResponsivePageSizeOptions) => {
  const rowHeight = options?.rowHeight ?? DEFAULT_ROW_HEIGHT;
  const headerHeight = options?.headerHeight ?? DEFAULT_HEADER_HEIGHT;
  const [container, setContainer] = useState<HTMLElement | null>(null);
  const [measuredPageSize, setMeasuredPageSize] = useState<number | null>(null);

  useEffect(() => {
    if (!container) return;

    const observer = new ResizeObserver(([entry]) => {
      setMeasuredPageSize(computePageSize(entry.contentRect.height, rowHeight, headerHeight));
    });
    observer.observe(container);
    return () => observer.disconnect();
  }, [container, rowHeight, headerHeight]);

  const containerRef = useCallback(
    (node: HTMLElement | null) => {
      setContainer(node);
      // Read on attach rather than from the observer's first (async) callback,
      // so the size is already known in the commit the container appears in.
      if (node) setMeasuredPageSize(computePageSize(node.clientHeight, rowHeight, headerHeight));
    },
    [rowHeight, headerHeight],
  );

  return {
    pageSize: measuredPageSize ?? MIN_PAGE_SIZE,
    isMeasured: measuredPageSize !== null,
    containerRef,
  };
};
