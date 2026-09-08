import React, { useState, type ReactNode } from 'react';
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
import { DataTableToolbar } from '@shared/presentation/ui/data-display/DataTableToolbar.tsx';
import type {
  DataTableFacet,
  FacetView,
} from '@shared/presentation/ui/data-display/DataTableFacetFilter.tsx';
import type { SelectionActionsProps } from '@shared/presentation/ui/controls/SelectionActions.tsx';

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

/** Diacritic-insensitive, so "reglage" matches "réglage". */
const normalize = (value: string) =>
  value
    .toLowerCase()
    .normalize('NFD')
    .replace(/\p{Diacritic}/gu, '');

/** Default haystack: every primitive field of the row — no coupling to the columns. */
const defaultSearchValues = (row: unknown): unknown[] =>
  row !== null && typeof row === 'object' ? Object.values(row) : [row];

const matchesQuery = (values: readonly unknown[], query: string) =>
  values.some(
    value =>
      (typeof value === 'string' || typeof value === 'number') &&
      normalize(String(value)).includes(query),
  );

/**
 * Resolves a facet against the rows: its options in display order, each with how many
 * rows carry it. Counts ignore the facet's own selection so the dropdown keeps showing
 * what selecting another value would yield.
 */
function buildFacetView<TData>(
  facet: DataTableFacet<TData>,
  rows: TData[],
  selected: string[],
): FacetView {
  const counts = new Map<string, number>();
  for (const row of rows) {
    const value = facet.value(row);
    if (value === undefined) continue;
    counts.set(value, (counts.get(value) ?? 0) + 1);
  }

  const options = facet.options
    ? facet.options
    : Array.from(counts.keys())
        .sort((a, b) => a.localeCompare(b))
        .map(value => ({ value, label: value }));

  return {
    id: facet.id,
    label: facet.label,
    // A declared option with no matching row still shows, at zero — the list of
    // categories should not jump around as the data changes.
    options: options.map(option => ({ ...option, count: counts.get(option.value) ?? 0 })),
    selected,
  };
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
  /**
   * Use a fixed table layout so the table honours its `w-full` width instead of
   * growing to fit cell content. Needed when a cell (e.g. expanded row content)
   * can be wider than the viewport and should scroll internally rather than
   * widening the whole table.
   */
  tableLayoutFixed?: boolean;
  /** Show the toolbar's search box. */
  searchable?: boolean;
  searchPlaceholder?: string;
  /**
   * What the search box matches against. Defaults to every primitive field of the row,
   * which covers most entities — pass this when the text to search is nested or computed.
   */
  searchValues?: (row: TData) => (string | number | null | undefined)[];
  /** Category filters, one dropdown each. Independent of {@link searchable}. */
  facets?: DataTableFacet<TData>[];
  /**
   * Bulk actions over the current selection. They live in the toolbar, next to the rows
   * they act on, and take it over while at least one row is selected.
   */
  selection?: SelectionActionsProps;
  /** Extra controls at the end of the toolbar; only rendered when there is a toolbar. */
  toolbarActions?: ReactNode;
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
  searchable = false,
  searchPlaceholder,
  searchValues,
  facets,
  selection,
  toolbarActions,
}: DataTableProps<TData, TValue>) {
  const [searchQuery, setSearchQuery] = useState('');
  const [facetSelection, setFacetSelection] = useState<Record<string, string[]>>({});

  const facetList = facets ?? [];
  const hasSelection = (selection?.selectedCount ?? 0) > 0;
  const showToolbar = searchable || facetList.length > 0 || hasSelection;
  const query = normalize(searchQuery.trim());

  // Filtering is client-side over the rows it was handed: on a server-paginated
  // list it narrows the current page, not the whole collection.
  const searchedRows = query
    ? data.filter(row =>
        matchesQuery(searchValues ? searchValues(row) : defaultSearchValues(row), query),
      )
    : data;

  const activeFacets = facetList.filter(facet => facetSelection[facet.id]?.length);
  const filteredData = activeFacets.length
    ? searchedRows.filter(row =>
        activeFacets.every(facet => {
          const value = facet.value(row);
          return value !== undefined && facetSelection[facet.id].includes(value);
        }),
      )
    : searchedRows;

  const facetViews = facetList.map(facet =>
    buildFacetView(facet, searchedRows, facetSelection[facet.id] ?? []),
  );

  const isFiltered = query.length > 0 || activeFacets.length > 0;

  const toggleFacetValue = (facetId: string, value: string) =>
    setFacetSelection(current => {
      const selected = current[facetId] ?? [];
      return {
        ...current,
        [facetId]: selected.includes(value)
          ? selected.filter(entry => entry !== value)
          : [...selected, value],
      };
    });

  const resetFilters = () => {
    setSearchQuery('');
    setFacetSelection({});
  };

  const table = useReactTable({
    data: filteredData,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getRowId,
  });

  return (
    <div className='flex h-full w-full flex-col overflow-hidden rounded-2xl border border-border/70 bg-card shadow-[0_1px_2px_oklch(0_0_0/0.04),0_12px_32px_-16px_oklch(0_0_0/0.18)]'>
      {showToolbar && (
        <DataTableToolbar
          searchable={searchable}
          searchPlaceholder={searchPlaceholder}
          searchQuery={searchQuery}
          onSearchChange={setSearchQuery}
          facets={facetViews}
          onToggleFacetValue={toggleFacetValue}
          onClearFacet={facetId => setFacetSelection(current => ({ ...current, [facetId]: [] }))}
          onReset={resetFilters}
          isFiltered={isFiltered}
          filteredCount={filteredData.length}
          totalCount={data.length}
          selection={selection && { selectableCount: data.length, ...selection }}
          actions={toolbarActions}
        />
      )}

      <div className='min-h-0 flex-1 overflow-auto'>
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
                        // Opaque on purpose: a translucent + `backdrop-blur` header creates one
                        // backdrop-filter layer per cell, re-rasterised every time the rows under
                        // it change — which is every keystroke in the search box.
                        'sticky top-0 z-10 h-11 bg-card px-5 text-[0.6875rem] font-semibold tracking-[0.08em] text-muted-foreground/90 uppercase',
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
                  {isFiltered ? (
                    <Trans>No result matches your filters.</Trans>
                  ) : (
                    <Trans>No results.</Trans>
                  )}
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </table>
      </div>
    </div>
  );
}
