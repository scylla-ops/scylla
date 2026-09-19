import type { Snippet } from 'svelte';
import { renderSnippet } from '@tanstack/svelte-table';
import type { DataTableColumn } from '@shared/presentation/ui-svelte';
import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
import type { SecretEntity } from '../../../domain/entities/secret.entity.ts';
import { secretMessages } from '../secret.messages.ts';

/** One snippet per column, each taking the row it renders. */
export interface SecretCells {
  name: Snippet<[SecretEntity]>;
  description: Snippet<[SecretEntity]>;
  createdAt: Snippet<[SecretEntity]>;
  actions: Snippet<[SecretEntity]>;
}

/**
 * The secrets table, as a column definition.
 *
 * Most of it survived the port untouched — `accessorKey`, `header`, `size`,
 * `minSize` are the same numbers the React version declared, which is the whole
 * dividend of TanStack Table being framework-agnostic data. Two things changed:
 *
 * - a `cell` used to return a React element and now returns a Svelte snippet,
 *   so the markup stays in `SecretList.svelte` and this file stays a `.ts`;
 * - centring moved from a wrapper `<div>` inside each cell to `meta.align`,
 *   which `DataTable` already reads for both the header and the cell.
 */
export const secretColumns = (cells: SecretCells): DataTableColumn<SecretEntity>[] => [
  {
    accessorKey: 'name',
    header: t(secretMessages.name),
    cell: ({ row }) => renderSnippet(cells.name, row.original),
    size: 240,
    minSize: 240,
    meta: { align: 'left' },
  },
  {
    accessorKey: 'description',
    header: t(secretMessages.description),
    cell: ({ row }) => renderSnippet(cells.description, row.original),
    size: 300,
    minSize: 180,
    meta: { align: 'center' },
  },
  {
    accessorKey: 'createdAt',
    header: t(secretMessages.created),
    cell: ({ row }) => renderSnippet(cells.createdAt, row.original),
    size: 180,
    minSize: 160,
    meta: { align: 'center' },
  },
  {
    id: 'actions',
    header: t(secretMessages.actions),
    cell: ({ row }) => renderSnippet(cells.actions, row.original),
    size: 100,
    minSize: 100,
    meta: { align: 'center' },
  },
];
