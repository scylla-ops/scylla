import { columnSizingFeature } from '@tanstack/table-core';
import type { CellData, ColumnDef, RowData, TableFeatures } from '@tanstack/table-core';

/**
 * The feature set `DataTable` builds its tables from.
 *
 * TanStack Table 9 composes features explicitly instead of shipping them all:
 * the core ones (rows, columns, headers, cells, the core row model) come for
 * free, everything else is opt-in. `columnSizingFeature` is here because it is
 * what declares `size` and `minSize` on a column definition — the two numbers
 * `buildGridTemplate` reads. Nothing else is opted into: no sorting, no
 * filtering, no pagination. `Pagination` is a separate, server-driven component.
 *
 * In particular `columnVisibilityFeature` is *not* here, which is why the
 * component iterates `row.getAllCells()` where the React one called
 * `getVisibleCells()` — no column in this app can be hidden, so the two return
 * the same list and the second costs a feature.
 */
export const DATA_TABLE_FEATURES = { columnSizingFeature } as const;

export type DataTableFeatures = typeof DATA_TABLE_FEATURES;

/**
 * A column definition for `DataTable`.
 *
 * Features carry types in v9, so a bare `ColumnDef<TData, TValue>` no longer
 * exists — the feature set is the first type argument. Aliasing it here means a
 * feature writes `DataTableColumn<SecretEntity>` and never has to name the
 * feature set.
 */
export type DataTableColumn<TData extends RowData, TValue extends CellData = CellData> = ColumnDef<
  DataTableFeatures,
  TData,
  TValue
>;

export type ColumnAlign = 'left' | 'center' | 'right';

declare module '@tanstack/table-core' {
  /* eslint-disable @typescript-eslint/no-unused-vars */
  interface ColumnMeta<
    in out TFeatures extends TableFeatures,
    in out TData extends RowData,
    TValue extends CellData = CellData,
  > {
    align?: ColumnAlign;
  }
  /* eslint-enable @typescript-eslint/no-unused-vars */
}

export const alignClass: Record<ColumnAlign, string> = {
  left: 'text-left',
  center: 'text-center',
  right: 'text-right',
};

export const headAlignClass: Record<ColumnAlign, string> = {
  left: 'justify-start',
  center: 'justify-center',
  right: 'justify-end',
};

export const cellAlignClass: Record<ColumnAlign, string> = {
  left: '',
  center: 'flex items-center justify-center',
  right: 'flex items-center justify-end',
};

/**
 * One grid track per column.
 *
 * A CSS table cannot express this: `table-layout: fixed` ignores `min-width` on
 * cells entirely, and `auto` never shrinks a column below its content. `minmax()`
 * gives every column a real floor:
 *
 * - `size` set       → `minmax(minSize, size)` — grows up to `size`, shrinks back to
 *   `minSize`, i.e. down to nothing when no `minSize` is declared.
 * - `size` undefined → `minmax(minSize, 1fr)` — absorbs all the leftover space.
 *
 * When no column is flexible, each track becomes `${size}fr` instead so the table
 * still fills its container, sharing the width proportionally as a fixed table
 * layout would — otherwise the columns would leave a gap on the right.
 *
 * Read from the `columns` prop, never from `column.columnDef`: TanStack defaults
 * the latter to `size: 150, minSize: 20`, which makes "unspecified" unreadable.
 */
export const buildGridTemplate = (columns: { size?: number; minSize?: number }[]): string => {
  const hasFlexibleColumn = columns.some(column => column.size === undefined);

  return columns
    .map(({ size, minSize }) => {
      const min = `${minSize ?? 0}px`;
      if (size === undefined) return `minmax(${min}, 1fr)`;
      return `minmax(${min}, ${hasFlexibleColumn ? `${size}px` : `${size}fr`})`;
    })
    .join(' ');
};

/** Summed minimums: below this the tracks overflow and the table must scroll. */
export const minTableWidthOf = (columns: { minSize?: number }[]): number =>
  columns.reduce((total, column) => total + (column.minSize ?? 0), 0);
