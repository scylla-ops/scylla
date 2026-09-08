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

const cellAlignClass: Record<ColumnAlign, string> = {
  left: '',
  center: 'flex items-center justify-center',
  right: 'flex items-center justify-end',
};

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
  /**
   * Use a fixed table layout so the table honours its `w-full` width instead of
   * growing to fit cell content. Needed when a cell (e.g. expanded row content)
   * can be wider than the viewport and should scroll internally rather than
   * widening the whole table.
   */
  tableLayoutFixed?: boolean;
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
  tableLayoutFixed = false,
}: DataTableProps<TData, TValue>) {
  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getRowId,
  });

  return (
    <div className='h-full w-full overflow-auto rounded-2xl border border-border/70 bg-card shadow-[0_1px_2px_oklch(0_0_0/0.04),0_12px_32px_-16px_oklch(0_0_0/0.18)]'>
      {/* border-separate keeps the sticky header's hairline attached while scrolling —
          with border-collapse the browser drops cell borders on a sticky thead. */}
      <table
        className={cn(
          'w-full caption-bottom border-separate border-spacing-0 text-sm',
          tableLayoutFixed && 'table-fixed',
        )}
      >
        <TableHeader>
          {table.getHeaderGroups().map(headerGroup => (
            <TableRow key={headerGroup.id} className='hover:bg-transparent'>
              {headerGroup.headers.map(header => {
                const align =
                  header.column.columnDef.meta?.align ?? (alignColumnsCenter ? 'center' : 'left');
                return (
                  <TableHead
                    key={header.id}
                    style={{
                      width: header.getSize() !== 150 ? `${header.getSize()}px` : 'auto',
                      minWidth: header.column.columnDef.minSize
                        ? `${header.column.columnDef.minSize}px`
                        : undefined,
                    }}
                    className={cn(
                      'sticky top-0 z-10 h-11 bg-card/80 px-5 text-[0.6875rem] font-semibold tracking-[0.08em] text-muted-foreground/90 uppercase backdrop-blur-xl',
                      'shadow-[inset_0_-1px_0_0_var(--border)]',
                      alignClass[align],
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

        <TableBody>
          {table.getRowModel().rows?.length ? (
            table.getRowModel().rows.map(row => {
              const isSelected = isRowSelected?.(row.original) ?? false;
              const isExpanded = isRowExpanded?.(row.original) ?? false;

              return (
                <React.Fragment key={row.id}>
                  <TableRow
                    data-state={isSelected ? 'selected' : undefined}
                    onClick={() => onRowClick?.(row)}
                    className={cn(
                      'transition-colors duration-150',
                      onRowClick && 'cursor-pointer',
                      onRowClick && !isSelected && 'hover:bg-muted/40',
                      // Inset shadow rather than a left border: the accent bar appears
                      // without shifting the first column by its width.
                      isSelected &&
                        '[&>td]:bg-primary/[0.07] [&>td:first-child]:shadow-[inset_3px_0_0_0_var(--primary)] hover:[&>td]:bg-primary/[0.1]',
                    )}
                  >
                    {row.getVisibleCells().map((cell, index) => {
                      const header = table.getHeaderGroups()[0].headers[index];
                      const align =
                        cell.column.columnDef.meta?.align ?? (alignRowsCenter ? 'center' : 'left');
                      return (
                        <TableCell
                          key={cell.id}
                          style={{
                            width: header.getSize() !== 150 ? `${header.getSize()}px` : 'auto',
                            minWidth: header.column.columnDef.minSize
                              ? `${header.column.columnDef.minSize}px`
                              : undefined,
                          }}
                          className={cn('px-5 py-3.5', alignClass[align])}
                        >
                          <div className={cn(cellAlignClass[align])}>
                            {flexRender(cell.column.columnDef.cell, cell.getContext())}
                          </div>
                        </TableCell>
                      );
                    })}
                  </TableRow>

                  {isExpanded && expandedContent && (
                    <TableRow key={`${row.id}-expanded`} className='hover:bg-transparent'>
                      <TableCell colSpan={columns.length} className='bg-muted/30 p-0'>
                        {expandedContent(row)}
                      </TableCell>
                    </TableRow>
                  )}
                </React.Fragment>
              );
            })
          ) : (
            <TableRow className='hover:bg-transparent'>
              <TableCell
                colSpan={columns.length}
                className='h-28 text-center text-sm text-muted-foreground'
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
