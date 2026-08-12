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
    <div className='w-full h-full border border-border bg-card shadow-sm rounded-xl overflow-auto'>
      <table
        className={cn('w-full caption-bottom text-sm relative', tableLayoutFixed && 'table-fixed')}
      >
        <TableHeader className='bg-muted/70 sticky top-0 z-10'>
          {table.getHeaderGroups().map(headerGroup => (
            <TableRow key={headerGroup.id} className='hover:bg-transparent border-b border-border'>
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
                      'h-12 px-4 text-muted-foreground font-semibold bg-muted/70 text-xs uppercase tracking-wider',
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
                      'border-b [&>td:first-child]:border-l-[3px] border-border/70 [&>td:first-child]:border-l-transparent transition-colors duration-150',
                      onRowClick && 'cursor-pointer',
                      onRowClick && !isSelected && 'hover:bg-muted/50',
                      isSelected &&
                        '[&>td]:bg-primary/6 [&>td:first-child]:border-l-[3px] [&>td:first-child]:border-l-primary hover:bg-primary/9',
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
                          className={cn('px-4 py-4', alignClass[align])}
                        >
                          <div className={cn(cellAlignClass[align])}>
                            {flexRender(cell.column.columnDef.cell, cell.getContext())}
                          </div>
                        </TableCell>
                      );
                    })}
                  </TableRow>

                  {isExpanded && expandedContent && (
                    <TableRow
                      key={`${row.id}-expanded`}
                      className='border-b border-border bg-muted/50'
                    >
                      <TableCell colSpan={columns.length} className='p-0'>
                        {expandedContent(row)}
                      </TableCell>
                    </TableRow>
                  )}
                </React.Fragment>
              );
            })
          ) : (
            <TableRow>
              <TableCell
                colSpan={columns.length}
                className='h-24 text-center text-muted-foreground'
              >
                No results.
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </table>
    </div>
  );
}
