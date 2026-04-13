import React from 'react';
import {
  flexRender,
  getCoreRowModel,
  useReactTable,
  type ColumnDef,
  type Row,
} from '@tanstack/react-table';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/modules/shared/presentation/ui/shadcn/table';
import { cn } from '@shared/presentation/utils';

interface DataTableProps<TData, TValue> {
  columns: ColumnDef<TData, TValue>[];
  data: TData[];
  onRowClick?: (row: Row<TData>) => void;
  getRowId?: (row: TData, index: number) => string;
  isRowSelected?: (row: TData) => boolean;
  expandedContent?: (row: Row<TData>) => React.ReactNode;
  isRowExpanded?: (row: TData) => boolean;
}

export function DataTable<TData, TValue>({
  columns,
  data,
  onRowClick,
  getRowId,
  isRowSelected,
  expandedContent,
  isRowExpanded,
}: DataTableProps<TData, TValue>) {
  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getRowId,
  });

  return (
    <div className='flex flex-col rounded-xl border border-slate-200 bg-white shadow-sm overflow-hidden'>
      <div className='overflow-x-auto'>
        <Table className='w-full'>
          <TableHeader className='sticky top-0 z-20 bg-slate-50'>
            {table.getHeaderGroups().map(headerGroup => (
              <TableRow key={headerGroup.id} className='hover:bg-transparent border-b border-slate-200'>
                {headerGroup.headers.map(header => (
                  <TableHead
                    key={header.id}
                    style={{
                      width: header.getSize() !== 150 ? `${header.getSize()}px` : 'auto',
                      minWidth: header.column.columnDef.minSize
                        ? `${header.column.columnDef.minSize}px`
                        : undefined,
                    }}
                    className='h-12 px-4 text-slate-600 font-semibold bg-slate-50'
                  >
                    {header.isPlaceholder
                      ? null
                      : flexRender(header.column.columnDef.header, header.getContext())}
                  </TableHead>
                ))}
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
                        'border-b border-slate-100 transition-all duration-200',
                        onRowClick && 'cursor-pointer hover:bg-slate-50 hover:shadow-sm',
                        isSelected && 'bg-blue-50 hover:bg-blue-100 border-blue-200',
                      )}
                    >
                      {row.getVisibleCells().map(cell => (
                        <TableCell
                          key={cell.id}
                          className='px-4 py-4'
                        >
                          {flexRender(cell.column.columnDef.cell, cell.getContext())}
                        </TableCell>
                      ))}
                    </TableRow>

                    {isExpanded && expandedContent && (
                      <TableRow key={`${row.id}-expanded`} className='border-b border-slate-200 bg-slate-50/50'>
                        <TableCell
                          colSpan={columns.length}
                          className='p-0'
                        >
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
                  className='h-24 text-center text-slate-500'
                >
                  No results.
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  );
}
