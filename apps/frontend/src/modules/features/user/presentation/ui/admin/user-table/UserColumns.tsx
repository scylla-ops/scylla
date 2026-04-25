import type { ColumnDef } from '@tanstack/react-table';
import type { User } from '@/modules/features/user/domain/models/user.model.ts';
import { Eye } from 'lucide-react';
import { Trans } from '@lingui/react/macro';
import { formatDate } from '@shared/utils/dateUtils.ts';
import { Avatar, AvatarFallback, AvatarImage } from '@shadcn';

type PipelineColumnMeta = {
  onView: (userId: string) => void;
};

export const createUserColumns = (meta: PipelineColumnMeta): ColumnDef<User>[] => [
  {
    id: 'username',
    header: () => (
      <div className={'flex w-full text-xs font-semibold uppercase tracking-wider'}>
        <Trans>User</Trans>
      </div>
    ),
    cell: ({ row }) => (
      <div className={'flex flexrow w-full h-full items-center gap-2'}>
        <Avatar className='h-8 w-8 rounded-lg'>
          <AvatarImage />
          <AvatarFallback className='rounded-lg'>
            {row.original.username.at(0)?.toUpperCase()}
          </AvatarFallback>
        </Avatar>
        <span>{row.original.username}</span>
      </div>
    ),
  },
  {
    id: 'creationDate',
    header: () => (
      <div className={'w-full text-center text-xs font-semibold uppercase tracking-wider'}>
        <Trans>Created at</Trans>
      </div>
    ),
    cell: ({ row }) => (
      <div className={'w-full text-center'}>
        <span>{formatDate(row.original.createdAt)}</span>
      </div>
    ),
  },
  {
    id: 'actions',
    header: () => (
      <div className={'w-full text-center text-xs font-semibold uppercase tracking-wider'}>
        <Trans>Actions</Trans>
      </div>
    ),
    cell: ({ row }) => (
      <div
        onClick={e => {
          e.stopPropagation();
          meta.onView(row.original.userId);
        }}
        className={'flex items-center justify-center gap-2'}
      >
        <Eye className='h-4 w-4 hover:scale-125 hover:text-primary transition-all cursor-pointer' />
      </div>
    ),
  },
];
