import type { ColumnDef } from '@tanstack/react-table';
import type { JobResponse } from '@/generated/job.ts';
import { JobStatus } from './JobStatus';
import { JobTimeline } from './JobTimeline';
import { JobActions } from './JobActions';
import { JobIdCell } from './JobIdCell';
import { calculateDuration, formatDuration, getRelativeTime } from '@shared/utils/date-utils.ts';
import { Trans } from '@lingui/react/macro';

type JobColumnMeta = {
  pipelineId: string;
  onDelete: (jobId: string) => void;
  onView: (jobId: string) => void;
};

export function createJobColumns(meta: JobColumnMeta): ColumnDef<JobResponse>[] {
  return [
    {
      accessorKey: 'status',
      header: () => (
        <div className={'flex items-center justify-center'}>
          <span className='text-xs font-semibold uppercase tracking-wider'>
            <Trans>Status</Trans>
          </span>
        </div>
      ),
      cell: ({ row }) => (
        <div className={'flex items-center justify-center'}>
          <JobStatus job={row.original} />{' '}
        </div>
      ),
      size: 160,
      minSize: 140,
    },
    {
      accessorKey: 'jobId',
      header: () => (
        <div className={'flex items-center justify-center'}>
          <span className='text-xs font-semibold uppercase tracking-wider'>
            <Trans>Job ID</Trans>
          </span>
        </div>
      ),
      cell: ({ row }) => (
        <div className={'flex items-center justify-center'}>
          <JobIdCell job={row.original} />
        </div>
      ),
      size: 180,
      minSize: 150,
    },
    {
      id: 'timeline',
      header: () => (
        <div className={'flex items-center justify-center'}>
          <span className='text-xs font-semibold uppercase tracking-wider'>
            <Trans>Timeline</Trans>
          </span>
        </div>
      ),
      cell: ({ row }) => (
        <div className={'flex items-center justify-center'}>
          <JobTimeline nodeExecutions={row.original.nodeExecutions} />
        </div>
      ),
      size: undefined, // Prend l'espace restant
      minSize: 200,
    },
    {
      id: 'duration',
      header: () => (
        <div className={'flex items-center justify-center'}>
          <span className='text-xs font-semibold uppercase tracking-wider'>
            <Trans>Duration</Trans>
          </span>
        </div>
      ),
      cell: ({ row }) => {
        const duration = calculateDuration(row.original.createdAt, row.original.updatedAt);
        return (
          <div className={'flex items-center justify-center'}>
            <span className='text-sm font-medium whitespace-nowrap'>
              {formatDuration(duration)}
            </span>
          </div>
        );
      },
      size: 100,
      minSize: 80,
    },
    {
      accessorKey: 'createdAt',
      header: () => (
        <div className={'flex items-center justify-center'}>
          <span className='text-xs font-semibold uppercase tracking-wider'>
            <Trans>Created</Trans>
          </span>
        </div>
      ),
      cell: ({ row }) => (
        <div className={'flex items-center justify-center'}>
          <span className='text-sm whitespace-nowrap'>
            {getRelativeTime(row.original.createdAt)}
          </span>
        </div>
      ),
      size: 100,
      minSize: 80,
    },
    {
      id: 'actions',
      header: () => (
        <div className={'flex items-center justify-center'}>
          <span className='text-xs font-semibold uppercase tracking-wider'>
            <Trans>Actions</Trans>
          </span>
        </div>
      ),
      cell: ({ row }) => (
        <div className={'flex items-center justify-center gap-2'}>
          <JobActions
            onView={e => {
              e.stopPropagation();
              meta.onView(row.original.jobId);
            }}
            onDelete={e => {
              e.stopPropagation();
              meta.onDelete(row.original.jobId);
            }}
          />
        </div>
      ),
      size: 100,
      minSize: 80,
    },
  ];
}
