import type { ColumnDef } from '@tanstack/react-table';
import type { UserResponse } from '@/generated/user.ts';

export const createUserColumns = (): ColumnDef<UserResponse>[] => [
  {
    id: 'username',
    header: 'Username',
    cell: ({ row }) => <span>{row.original.username}</span>,
  },
  {
    id: 'creationDate',
    header: 'Created at',
    cell: ({ row }) => <span>{row.original.createdAt}</span>,
  },
  {
    id: 'actions',
    header: 'Actions',
  },
];
