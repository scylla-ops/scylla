import type { DataTableColumn } from './data-table.ts';

/** The row type `DataTable.fixture.svelte` pins the component to. */
export interface Datum extends Record<string, unknown> {
  id: string;
  name: string;
  status: string;
}

export const data: Datum[] = [
  { id: 'p-1', name: 'build', status: 'running' },
  { id: 'p-2', name: 'deploy', status: 'failed' },
];

export const columns: DataTableColumn<Datum>[] = [
  { id: 'name', header: 'Name', accessorKey: 'name' },
  { id: 'status', header: 'Status', accessorKey: 'status' },
];
