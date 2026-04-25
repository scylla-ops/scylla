import type { ColumnDef } from '@tanstack/react-table';
import type { User } from '@/modules/features/user/domain/models/user.model.ts';
import { Eye } from 'lucide-react';
import { Trans } from '@lingui/react/macro';
import { formatDate } from '@shared/utils/dateUtils.ts';

type PipelineColumnMeta = {
  onView: (userId: string) => void;
};

export const createUserColumns = (meta: PipelineColumnMeta): ColumnDef<User>[] => [
  {
    id: 'username',
    header: () => (
      <div className={'w-full text-center text-xs font-semibold uppercase tracking-wider'}>
        <Trans>Username</Trans>
      </div>
    ),
    cell: ({ row }) => (
      <div className={'w-full text-center'}>
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
