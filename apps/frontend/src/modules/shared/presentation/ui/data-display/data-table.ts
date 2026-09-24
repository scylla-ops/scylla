import { columnSizingFeature } from '@tanstack/table-core';
import type { CellData, ColumnDef, RowData, TableFeatures } from '@tanstack/table-core';

/**
 * The TanStack Table features `DataTable` uses. `columnSizingFeature` declares
 * `size` and `minSize`, read by `buildGridTemplate`. No column can be hidden.
 */
export const DATA_TABLE_FEATURES = { columnSizingFeature } as const;

export type DataTableFeatures = typeof DATA_TABLE_FEATURES;

/** A column definition, with the feature set already filled in. */
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
 * One grid track per column: `minmax(minSize, size)`, or `minmax(minSize, 1fr)`
 * without a `size`. When no column is flexible, `${size}fr` so the table still
 * fills its container.
 *
 * Read from the `columns` prop: `column.columnDef` defaults `size` to 150.
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

/** Below this width the table scrolls. */
export const minTableWidthOf = (columns: { minSize?: number }[]): number =>
  columns.reduce((total, column) => total + (column.minSize ?? 0), 0);
