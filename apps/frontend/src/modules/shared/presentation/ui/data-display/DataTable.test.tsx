import { describe, it, expect, vi } from 'vitest';
import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { ColumnDef } from '@tanstack/react-table';
import { renderWithI18n } from '@/test/render.tsx';
import { DataTable } from './DataTable';

interface Row {
  id: string;
  name: string;
  status: string;
}

const rows: Row[] = [
  { id: 'p-1', name: 'build', status: 'running' },
  { id: 'p-2', name: 'deploy', status: 'failed' },
];

const columns: ColumnDef<Row, unknown>[] = [
  { id: 'name', header: 'Name', accessorKey: 'name' },
  { id: 'status', header: 'Status', accessorKey: 'status' },
];

/** The header row and the body rows share one grid template — read it off a row. */
const gridTemplateOf = (row: HTMLElement) => row.style.gridTemplateColumns;

describe('DataTable', () => {
  it('renders one column header per column and one row per datum', () => {
    renderWithI18n(<DataTable columns={columns} data={rows} />);

    expect(screen.getAllByRole('columnheader').map(h => h.textContent)).toEqual(['Name', 'Status']);
    expect(screen.getByText('build')).toBeInTheDocument();
    expect(screen.getByText('deploy')).toBeInTheDocument();
  });

  it('shows a single "No results." row for an empty dataset, not an empty body', () => {
    renderWithI18n(<DataTable columns={columns} data={[]} />);

    expect(screen.getByText('No results.')).toBeInTheDocument();
    // The header row is still there; the body contributes exactly one.
    expect(screen.getAllByRole('row')).toHaveLength(2);
  });

  it('calls onRowClick with the clicked row, carrying the original datum', async () => {
    const onRowClick = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<DataTable columns={columns} data={rows} onRowClick={onRowClick} />);

    await user.click(screen.getByText('deploy'));

    expect(onRowClick).toHaveBeenCalledTimes(1);
    expect(onRowClick.mock.calls[0][0].original).toEqual(rows[1]);
  });

  it('does not blow up on a click when no onRowClick was given', async () => {
    const user = userEvent.setup();
    renderWithI18n(<DataTable columns={columns} data={rows} />);

    await user.click(screen.getByText('deploy'));
    expect(screen.getByText('deploy')).toBeInTheDocument();
  });

  it('marks only the rows isRowSelected accepts, via data-state', () => {
    renderWithI18n(
      <DataTable columns={columns} data={rows} isRowSelected={row => row.id === 'p-2'} />,
    );

    const [, first, second] = screen.getAllByRole('row'); // [header, ...body]
    expect(first).not.toHaveAttribute('data-state', 'selected');
    expect(second).toHaveAttribute('data-state', 'selected');
  });

  it('renders expandedContent as an extra row, and only for the expanded ones', () => {
    renderWithI18n(
      <DataTable
        columns={columns}
        data={rows}
        isRowExpanded={row => row.id === 'p-1'}
        expandedContent={row => <span>logs for {row.original.name}</span>}
      />,
    );

    expect(screen.getByText(/logs for/)).toHaveTextContent('logs for build');
    expect(screen.queryByText('logs for deploy')).not.toBeInTheDocument();
    // header + 2 data rows + 1 expansion row
    expect(screen.getAllByRole('row')).toHaveLength(4);
  });

  it('ignores isRowExpanded when no expandedContent renderer was given', () => {
    renderWithI18n(<DataTable columns={columns} data={rows} isRowExpanded={() => true} />);
    expect(screen.getAllByRole('row')).toHaveLength(3);
  });

  it('uses getRowId so rows keep their identity across a reorder', () => {
    const { rerender } = renderWithI18n(
      <DataTable columns={columns} data={rows} getRowId={row => row.id} />,
    );
    expect(screen.getAllByRole('row')[1]).toHaveTextContent('build');

    rerender(<DataTable columns={columns} data={[...rows].reverse()} getRowId={row => row.id} />);
    expect(screen.getAllByRole('row')[1]).toHaveTextContent('deploy');
  });

  describe('grid template', () => {
    it('a column with no size absorbs the leftover space as 1fr, floored at its minSize', () => {
      renderWithI18n(
        <DataTable
          columns={[{ id: 'name', header: 'Name', accessorKey: 'name', minSize: 120 }]}
          data={rows}
        />,
      );

      expect(gridTemplateOf(screen.getAllByRole('row')[0])).toBe('minmax(120px, 1fr)');
    });

    it('a column with no minSize can shrink to nothing', () => {
      renderWithI18n(
        <DataTable columns={[{ id: 'name', header: 'Name', accessorKey: 'name' }]} data={rows} />,
      );

      expect(gridTemplateOf(screen.getAllByRole('row')[0])).toBe('minmax(0px, 1fr)');
    });

    it('sized columns stay in px while at least one column is flexible', () => {
      renderWithI18n(
        <DataTable
          columns={[
            { id: 'name', header: 'Name', accessorKey: 'name', size: 200, minSize: 80 },
            { id: 'status', header: 'Status', accessorKey: 'status' },
          ]}
          data={rows}
        />,
      );

      expect(gridTemplateOf(screen.getAllByRole('row')[0])).toBe(
        'minmax(80px, 200px) minmax(0px, 1fr)',
      );
    });

    it('with every column sized, the tracks become fr so the table still fills its container', () => {
      renderWithI18n(
        <DataTable
          columns={[
            { id: 'name', header: 'Name', accessorKey: 'name', size: 2, minSize: 80 },
            { id: 'status', header: 'Status', accessorKey: 'status', size: 1 },
          ]}
          data={rows}
        />,
      );

      expect(gridTemplateOf(screen.getAllByRole('row')[0])).toBe(
        'minmax(80px, 2fr) minmax(0px, 1fr)',
      );
    });

    it('reads sizes from the columns prop, not from TanStack\'s size: 150 / minSize: 20 defaults', () => {
      renderWithI18n(<DataTable columns={columns} data={rows} />);
      expect(gridTemplateOf(screen.getAllByRole('row')[0])).toBe(
        'minmax(0px, 1fr) minmax(0px, 1fr)',
      );
    });

    it('pins the table to the summed minimums so the sticky header tracks a horizontal scroll', () => {
      renderWithI18n(
        <DataTable
          columns={[
            { id: 'name', header: 'Name', accessorKey: 'name', minSize: 120 },
            { id: 'status', header: 'Status', accessorKey: 'status', minSize: 80 },
          ]}
          data={rows}
        />,
      );

      expect(screen.getByRole('table')).toHaveStyle({ minWidth: '200px' });
    });

    it('leaves minWidth unset when no column declares a minimum', () => {
      renderWithI18n(<DataTable columns={columns} data={rows} />);
      expect(screen.getByRole('table').style.minWidth).toBe('');
    });
  });

  describe('alignment', () => {
    it("a column's meta.align wins over the table-wide default", () => {
      renderWithI18n(
        <DataTable
          columns={[
            { id: 'name', header: 'Name', accessorKey: 'name', meta: { align: 'right' } },
            { id: 'status', header: 'Status', accessorKey: 'status' },
          ]}
          data={rows}
          alignColumnsCenter
        />,
      );

      const [name, status] = screen.getAllByRole('columnheader');
      expect(name).toHaveClass('text-right', 'justify-end');
      expect(status).toHaveClass('text-center', 'justify-center');
    });

    it('alignRowsCenter centres the cells without touching the headers', () => {
      renderWithI18n(<DataTable columns={columns} data={rows} alignRowsCenter />);

      const firstRow = screen.getAllByRole('row')[1];
      expect(within(firstRow).getAllByRole('cell')[0]).toHaveClass('text-center');
      expect(screen.getAllByRole('columnheader')[0]).toHaveClass('text-left');
    });
  });
});
