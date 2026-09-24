import type { Snippet } from 'svelte';
import { renderSnippet } from '@tanstack/svelte-table';
import type { DataTableColumn } from '@shared/presentation/ui';
import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
import type { UserEntity } from '../../../../domain/entities/user.entity.ts';
import { userMessages } from '../../user.messages.ts';

export interface UserCells {
  username: Snippet<[UserEntity]>;
  creationDate: Snippet<[UserEntity]>;
  actions: Snippet<[UserEntity]>;
}

/**
 * The directory table.
 *
 * Sizes and ids are the React version's; the centring that its headers did with
 * a wrapper `<div>` is now `meta.align`, which `DataTable` applies to the header
 * and the cell alike.
 */
export const userColumns = (cells: UserCells): DataTableColumn<UserEntity>[] => [
  {
    id: 'username',
    header: t(userMessages.user),
    cell: ({ row }) => renderSnippet(cells.username, row.original),
    size: 200,
    minSize: 200,
    meta: { align: 'left' },
  },
  {
    id: 'creationDate',
    header: t(userMessages.createdAt),
    cell: ({ row }) => renderSnippet(cells.creationDate, row.original),
    size: 250,
    minSize: 150,
    meta: { align: 'center' },
  },
  {
    id: 'actions',
    header: t(userMessages.actions),
    cell: ({ row }) => renderSnippet(cells.actions, row.original),
    // No minSize: the first column to give its width back on a narrow viewport.
    size: 100,
    meta: { align: 'center' },
  },
];
