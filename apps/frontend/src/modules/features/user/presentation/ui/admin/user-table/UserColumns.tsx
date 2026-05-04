import type { ColumnDef } from '@tanstack/react-table';
import type { User } from '@/modules/features/user/domain/models/user.model.ts';
import { Eye } from 'lucide-react';
import { Trans } from '@lingui/react/macro';
import { formatDate } from '@shared/utils/date-utils.ts';
import { Avatar, AvatarFallback, AvatarImage, Button } from '@shadcn';
import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn/tooltip.tsx';

type PipelineColumnMeta = {
  onView: (userId: string) => void;
};

export const createUserColumns = (meta: PipelineColumnMeta): ColumnDef<User>[] => [
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
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            size={'icon'}
            className={
              'h-8 w-8 cursor-pointer transition-all hover:scale-125 hover:text-primary hover:bg-primary-hover rounded-full'
            }
            variant='ghost'
            onSelect={e => {
              e.preventDefault();
              meta.onView(row.original.userId);
            }}
          >
            <Eye />
          </Button>
        </TooltipTrigger>
        <TooltipContent>
          <p>
            <Trans>View</Trans>
          </p>
        </TooltipContent>
      </Tooltip>
    ),
  },
];
