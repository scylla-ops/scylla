import type { ColumnDef } from '@tanstack/react-table';
import type { UserEntity } from '@/modules/features/user/domain/entities/user.entity.ts';
import { Eye } from 'lucide-react';
import { Trans } from '@lingui/react/macro';
import { formatDate } from '@shared/utils/date-utils.ts';
import { Avatar, AvatarFallback, AvatarImage } from '@shadcn';
import { IconButton } from '@shared/presentation/ui';

type PipelineColumnMeta = {
  onView: (userId: string) => void;
};

export const createUserColumns = (meta: PipelineColumnMeta): ColumnDef<UserEntity>[] => [
  {
    id: 'username',
    header: () => <Trans>User</Trans>,
    cell: ({ row }) => (
      <div className='flex flex-row items-center gap-2'>
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
    header: () => <Trans>Created at</Trans>,
    cell: ({ row }) => <span>{formatDate(row.original.createdAt)}</span>,
  },
  {
    id: 'actions',
    header: () => <Trans>Actions</Trans>,
    cell: ({ row }) => (
      <IconButton
        icon={Eye}
        tooltip={<Trans>View</Trans>}
        onClick={e => {
          e.preventDefault();
          meta.onView(row.original.userId);
        }}
      />
    ),
  },
];
