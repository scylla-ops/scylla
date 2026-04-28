import type { ColumnDef } from '@tanstack/react-table';
import type { Worker } from '@/modules/features/workers/domain/models/worker.model.ts';
import { Eye } from 'lucide-react';
import { Trans } from '@lingui/react/macro';
import { formatDate } from '@shared/utils/date-utils.ts';
import { Badge } from '@shadcn';

type WorkersColumnMeta = {
  onView: (workerId: string) => void;
};

// Helper to determine status color
const getStatusColor = (status: string) => {
  const statusLower = status.toLowerCase();
  if (statusLower === 'connected') return 'bg-green-500';
  if (statusLower === 'disconnected') return 'bg-gray-500';
  if (statusLower === 'error') return 'bg-red-500';
  return 'bg-yellow-500';
};

export const createWorkerColumns = (meta: WorkersColumnMeta): ColumnDef<Worker>[] => [
  {
    id: 'hostname',
    header: () => (
      <div className={'flex w-full text-xs font-semibold uppercase tracking-wider'}>
        <Trans>Hostname</Trans>
      </div>
    ),
    cell: ({ row }) => <div className='font-medium'>{row.original.hostname}</div>,
  },
  {
    id: 'agentId',
    header: () => (
      <div className={'flex w-full text-xs font-semibold uppercase tracking-wider'}>
        <Trans>Agent ID</Trans>
      </div>
    ),
    cell: ({ row }) => (
      <div className='text-sm text-muted-foreground font-mono'>
        {row.original.agentId.substring(0, 26)}
        {row.original.agentId.length > 26 ? '...' : ''}
      </div>
    ),
  },
  {
    id: 'status',
    header: () => (
      <div className={'flex w-full text-xs font-semibold uppercase tracking-wider'}>
        <Trans>Status</Trans>
      </div>
    ),
    cell: ({ row }) => (
      <Badge className={getStatusColor(row.original.status)}>{row.original.status}</Badge>
    ),
  },
  {
    id: 'lastSeenAt',
    header: () => (
      <div className={'w-full text-center text-xs font-semibold uppercase tracking-wider'}>
        <Trans>Last Seen</Trans>
      </div>
    ),
    cell: ({ row }) => (
      <div className={'w-full text-center'}>
        <span className='text-sm'>{formatDate(row.original.lastSeenAt)}</span>
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
      <div className={'flex items-center justify-center gap-2'} onClick={e => e.stopPropagation()}>
        <Eye
          className='h-4 w-4 hover:scale-125 hover:text-primary transition-all cursor-pointer'
          onClick={() => meta.onView(row.original.agentId)}
        />
      </div>
    ),
  },
];
