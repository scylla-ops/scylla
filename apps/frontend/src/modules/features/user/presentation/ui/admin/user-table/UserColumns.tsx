import type { ColumnDef } from '@tanstack/react-table';
import type { User } from '@/modules/features/user/domain/models/user.model.ts';

export const createUserColumns = (): ColumnDef<User>[] => [
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
