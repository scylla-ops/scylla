export const MIN_PAGE_SIZE = 5;
export const MAX_PAGE_SIZE = 50;
export const DEFAULT_ROW_HEIGHT = 61;
export const DEFAULT_HEADER_HEIGHT = 44;

/**
 * How many rows fit in the room the layout left, clamped to a sane range.
 *
 * Pure on purpose: it is the whole of the responsive-page-size rule, and it is
 * tested without a DOM. The measuring is a separate concern — see
 * `measured-height.svelte.ts`.
 */
export const computePageSize = (
  containerHeight: number,
  rowHeight: number = DEFAULT_ROW_HEIGHT,
  headerHeight: number = DEFAULT_HEADER_HEIGHT,
): number => {
  const rows = Math.floor((containerHeight - headerHeight) / rowHeight);
  return Math.min(MAX_PAGE_SIZE, Math.max(MIN_PAGE_SIZE, rows));
};
