import React from 'react';
import {
  flexRender,
  getCoreRowModel,
  useReactTable,
  type ColumnDef,
  type Row,
} from '@tanstack/react-table';
import {
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/modules/shared/presentation/ui/shadcn/table';
import { cn } from '@shared/presentation/utils';
import { Trans } from '@lingui/react/macro';

type ColumnAlign = 'left' | 'center' | 'right';

declare module '@tanstack/react-table' {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  interface ColumnMeta<TData, TValue> {
    align?: ColumnAlign;
  }
}

const alignClass: Record<ColumnAlign, string> = {
  left: 'text-left',
  center: 'text-center',
  right: 'text-right',
};

const headAlignClass: Record<ColumnAlign, string> = {
  left: 'justify-start',
  center: 'justify-center',
  right: 'justify-end',
};

const cellAlignClass: Record<ColumnAlign, string> = {
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
function buildGridTemplate(columns: { size?: number; minSize?: number }[]): string {
  const hasFlexibleColumn = columns.some(column => column.size === undefined);

  return columns
    .map(({ size, minSize }) => {
      const min = `${minSize ?? 0}px`;
      if (size === undefined) return `minmax(${min}, 1fr)`;
      return `minmax(${min}, ${hasFlexibleColumn ? `${size}px` : `${size}fr`})`;
    })
    .join(' ');
}

interface DataTableProps<TData, TValue> {
  columns: ColumnDef<TData, TValue>[];
  data: TData[];
  onRowClick?: (row: Row<TData>) => void;
  getRowId?: (row: TData, index: number) => string;
  isRowSelected?: (row: TData) => boolean;
  expandedContent?: (row: Row<TData>) => React.ReactNode;
  isRowExpanded?: (row: TData) => boolean;
  alignColumnsCenter?: boolean;
  alignRowsCenter?: boolean;
}

export function DataTable<TData, TValue>({
  columns,
  data,
  onRowClick,
  getRowId,
  isRowSelected,
  expandedContent,
  isRowExpanded,
  alignRowsCenter = false,
  alignColumnsCenter = false,
}: DataTableProps<TData, TValue>) {
  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getRowId,
  });

  const gridTemplateColumns = buildGridTemplate(columns);
  // Below the summed minimums the tracks overflow the container; pinning the table
  // to that width keeps the header and row backgrounds under them while scrolling.
  const minTableWidth = columns.reduce((total, column) => total + (column.minSize ?? 0), 0);

  return (
    <div className='h-full w-full overflow-auto rounded-2xl border border-border/70 bg-card shadow-[0_1px_2px_oklch(0_0_0/0.04),0_12px_32px_-16px_oklch(0_0_0/0.18)]'>
      {/* Rows are CSS grids, not table rows — see buildGridTemplate. The table,
          thead and tbody stay plain blocks so the sticky header keeps the whole
          table as its containing block (a grid item is trapped in its own area).
          `display` other than `table-*` drops the implicit ARIA roles: restate them. */}
      <table
        role='table'
        className='block w-full text-sm'
        style={{ minWidth: minTableWidth > 0 ? `${minTableWidth}px` : undefined }}
      >
        <TableHeader
          role='rowgroup'
          className='sticky top-0 z-10 block bg-card/80 shadow-[inset_0_-1px_0_0_var(--border)] backdrop-blur-xl'
        >
          {table.getHeaderGroups().map(headerGroup => (
            <TableRow
              key={headerGroup.id}
              role='row'
              className='grid hover:bg-transparent'
              style={{ gridTemplateColumns }}
            >
              {headerGroup.headers.map(header => {
                const align =
                  header.column.columnDef.meta?.align ?? (alignColumnsCenter ? 'center' : 'left');
                return (
                  <TableHead
                    key={header.id}
                    role='columnheader'
                    className={cn(
                      'flex h-11 min-w-0 items-center overflow-hidden px-5 text-[0.6875rem] font-semibold tracking-[0.08em] text-muted-foreground/90 uppercase',
                      alignClass[align],
                      headAlignClass[align],
                    )}
                  >
                    {header.isPlaceholder
                      ? null
                      : flexRender(header.column.columnDef.header, header.getContext())}
                  </TableHead>
                );
              })}
            </TableRow>
          ))}
        </TableHeader>

        <TableBody role='rowgroup' className='block'>
          {table.getRowModel().rows?.length ? (
            table.getRowModel().rows.map(row => {
              const isSelected = isRowSelected?.(row.original) ?? false;
              const isExpanded = isRowExpanded?.(row.original) ?? false;

              return (
                <React.Fragment key={row.id}>
                  <TableRow
                    role='row'
                    data-state={isSelected ? 'selected' : undefined}
                    onClick={() => onRowClick?.(row)}
                    style={{ gridTemplateColumns }}
                    className={cn(
                      'grid transition-colors duration-150',
                      onRowClick && 'cursor-pointer',
                      onRowClick && !isSelected && 'hover:bg-muted/40',
                      // Inset shadow rather than a left border: the accent bar appears
                      // without shifting the first column by its width.
                      isSelected &&
                        '[&>td]:bg-primary/[0.07] [&>td:first-child]:shadow-[inset_3px_0_0_0_var(--primary)] hover:[&>td]:bg-primary/[0.1]',
                    )}
                  >
                    {row.getVisibleCells().map(cell => {
                      const align =
                        cell.column.columnDef.meta?.align ?? (alignRowsCenter ? 'center' : 'left');
                      return (
                        <TableCell
                          key={cell.id}
                          role='cell'
                          className={cn(
                            'flex min-w-0 items-center overflow-hidden px-5 py-3.5',
                            alignClass[align],
                          )}
                        >
                          <div className={cn('w-full min-w-0', cellAlignClass[align])}>
                            {flexRender(cell.column.columnDef.cell, cell.getContext())}
                          </div>
                        </TableCell>
                      );
                    })}
                  </TableRow>

                  {isExpanded && expandedContent && (
                    <TableRow
                      role='row'
                      className='grid grid-cols-[minmax(0,1fr)] hover:bg-transparent'
                    >
                      <TableCell role='cell' className='min-w-0 overflow-hidden bg-muted/30 p-0'>
                        {expandedContent(row)}
                      </TableCell>
                    </TableRow>
                  )}
                </React.Fragment>
              );
            })
          ) : (
            <TableRow role='row' className='grid grid-cols-[minmax(0,1fr)] hover:bg-transparent'>
              <TableCell
                role='cell'
                className='flex h-28 items-center justify-center text-sm text-muted-foreground'
              >
                <Trans>No results.</Trans>
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </table>
    </div>
  );
}
