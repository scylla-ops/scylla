import { useMeasuredHeight } from '@shared/presentation/hooks/use-measured-height.ts';

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
  const { height, containerRef } = useMeasuredHeight();

  const measuredPageSize =
    height === null ? null : computePageSize(height, rowHeight, headerHeight);

  return {
    pageSize: measuredPageSize ?? MIN_PAGE_SIZE,
    isMeasured: measuredPageSize !== null,
    containerRef,
  };
};
