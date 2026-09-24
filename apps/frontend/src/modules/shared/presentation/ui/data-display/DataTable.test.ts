import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { createRawSnippet } from 'svelte';
import type { Row } from '@tanstack/table-core';
import { render } from '@/test/render.svelte.ts';
// The fixture, not the component: it is what pins the generic row type. See
// the comment at the top of `DataTable.fixture.svelte`.
import DataTable from './DataTable.fixture.svelte';
import { columns, data, type Datum } from './data-table.fixture.ts';
import type { DataTableColumn, DataTableFeatures } from './data-table.ts';

/** The header row and the body rows share one grid template — read it off a row. */
const gridTemplateOf = (row: HTMLElement) => row.style.gridTemplateColumns;

const logsSnippet = createRawSnippet<[Row<DataTableFeatures, Datum>]>(row => ({
  render: () => `<span>logs for ${row().original.name}</span>`,
}));

describe('DataTable', () => {
  it('renders one column header per column and one row per datum', () => {
    render(DataTable, { columns, data });

    expect(screen.getAllByRole('columnheader').map(h => h.textContent?.trim())).toEqual([
      'Name',
      'Status',
    ]);
    expect(screen.getByText('build')).toBeInTheDocument();
    expect(screen.getByText('deploy')).toBeInTheDocument();
  });

  it('shows a single "No results." row for an empty dataset, not an empty body', () => {
    render(DataTable, { columns, data: [] });

    expect(screen.getByText('No results.')).toBeInTheDocument();
    // The header row is still there; the body contributes exactly one.
    expect(screen.getAllByRole('row')).toHaveLength(2);
  });

  it('calls onRowClick with the clicked row, carrying the original datum', async () => {
    const onRowClick = vi.fn();
    render(DataTable, { columns, data, onRowClick });

    await userEvent.click(screen.getByText('deploy'));

    expect(onRowClick).toHaveBeenCalledOnce();
    expect(onRowClick.mock.calls[0][0].original).toEqual(data[1]);
  });

  it('does not blow up on a click when no onRowClick was given', async () => {
    render(DataTable, { columns, data });

    await userEvent.click(screen.getByText('deploy'));

    expect(screen.getByText('deploy')).toBeInTheDocument();
  });

  it('marks only the rows isRowSelected accepts, via data-state', () => {
    render(DataTable, { columns, data, isRowSelected: (row: Datum) => row.id === 'p-2' });

    const [, first, second] = screen.getAllByRole('row'); // [header, ...body]
    expect(first).not.toHaveAttribute('data-state', 'selected');
    expect(second).toHaveAttribute('data-state', 'selected');
  });

  it('renders expandedContent as an extra row, and only for the expanded ones', () => {
    render(DataTable, {
      columns,
      data,
      isRowExpanded: (row: Datum) => row.id === 'p-1',
      expandedContent: logsSnippet,
    });

    expect(screen.getByText(/logs for/)).toHaveTextContent('logs for build');
    expect(screen.queryByText('logs for deploy')).toBeNull();
    // header + 2 data rows + 1 expansion row
    expect(screen.getAllByRole('row')).toHaveLength(4);
  });

  it('ignores isRowExpanded when no expandedContent snippet was given', () => {
    render(DataTable, { columns, data, isRowExpanded: () => true });

    expect(screen.getAllByRole('row')).toHaveLength(3);
  });

  it('uses getRowId so rows keep their identity across a reorder', async () => {
    const getRowId = (row: Datum) => row.id;
    const { rerender } = render(DataTable, { columns, data, getRowId });
    expect(screen.getAllByRole('row')[1]).toHaveTextContent('build');

    await rerender({ columns, data: [...data].reverse(), getRowId });

    expect(screen.getAllByRole('row')[1]).toHaveTextContent('deploy');
  });

  it('puts the grid template on every row, header included', () => {
    render(DataTable, {
      columns: [
        { id: 'name', header: 'Name', accessorKey: 'name', size: 200, minSize: 80 },
        { id: 'status', header: 'Status', accessorKey: 'status' },
      ] satisfies DataTableColumn<Datum>[],
      data,
    });

    const expected = 'minmax(80px, 200px) minmax(0px, 1fr)';
    for (const row of screen.getAllByRole('row')) {
      expect(gridTemplateOf(row)).toBe(expected);
    }
  });

  it('pins the table to the summed minimums so the sticky header tracks a horizontal scroll', () => {
    render(DataTable, {
      columns: [
        { id: 'name', header: 'Name', accessorKey: 'name', minSize: 120 },
        { id: 'status', header: 'Status', accessorKey: 'status', minSize: 80 },
      ] satisfies DataTableColumn<Datum>[],
      data,
    });

    expect(screen.getByRole('table')).toHaveStyle({ minWidth: '200px' });
  });

  it('leaves min-width unset when no column declares a minimum', () => {
    render(DataTable, { columns, data });

    expect(screen.getByRole('table').style.minWidth).toBe('');
  });

  it("lets a column's meta.align win over the table-wide default", () => {
    render(DataTable, {
      columns: [
        { id: 'name', header: 'Name', accessorKey: 'name', meta: { align: 'right' } },
        { id: 'status', header: 'Status', accessorKey: 'status' },
      ] satisfies DataTableColumn<Datum>[],
      data,
      alignColumnsCenter: true,
    });

    const [name, status] = screen.getAllByRole('columnheader');
    expect(name).toHaveClass('text-right', 'justify-end');
    expect(status).toHaveClass('text-center', 'justify-center');
  });
});
