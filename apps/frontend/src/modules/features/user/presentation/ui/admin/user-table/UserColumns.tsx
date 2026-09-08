import type { ColumnDef } from '@tanstack/react-table';
import type { UserEntity } from '@/modules/features/user/domain/entities/user.entity.ts';
import { Eye } from 'lucide-react';
import { Trans } from '@lingui/react/macro';
import { formatDate } from '@shared/utils/date-utils.ts';
import { Avatar, AvatarFallback, AvatarImage } from '@shadcn';
import { IconButton, TruncatedText } from '@shared/presentation/ui';

type PipelineColumnMeta = {
  onView: (userId: string) => void;
};

export const createUserColumns = (meta: PipelineColumnMeta): ColumnDef<UserEntity>[] => [
  {
    id: 'username',
    header: () => <Trans>User</Trans>,
    cell: ({ row }) => (
      <div className='flex w-full min-w-0 flex-row items-center gap-2'>
        <Avatar className='h-8 w-8 shrink-0 rounded-lg'>
          <AvatarImage />
          <AvatarFallback className='rounded-lg'>
            <TruncatedText className='text-xs'>
              {row.original.username.at(0)?.toUpperCase()}
            </TruncatedText>
          </AvatarFallback>
        </Avatar>
        <span className='truncate'>{row.original.username}</span>
      </div>
    ),
    size: 200,
    minSize: 200,
  },
  {
    id: 'creationDate',
    header: () => <Trans>Created at</Trans>,
    cell: ({ row }) => <span className='truncate'>{formatDate(row.original.createdAt)}</span>,
    minSize: 150,
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
    // No minSize: the first column to give its width back on a narrow viewport.
    size: 100,
  },
];
