import type { Snippet } from 'svelte';
import { renderSnippet } from '@tanstack/svelte-table';
import type { DataTableColumn } from '@shared/presentation/ui';
import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
import type { SecretEntity } from '../../../domain/entities/secret.entity.ts';
import { secretMessages } from '../secret.messages.ts';

export interface SecretCells {
  name: Snippet<[SecretEntity]>;
  description: Snippet<[SecretEntity]>;
  createdAt: Snippet<[SecretEntity]>;
  actions: Snippet<[SecretEntity]>;
}

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
