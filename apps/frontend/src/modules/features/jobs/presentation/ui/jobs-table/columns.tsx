import type { ColumnDef } from '@tanstack/react-table';
import type { JobResponse } from '@/generated/job.ts';
import { JobStatus } from './JobStatus';
import { JobTimeline } from './JobTimeline';
import { JobActions } from './JobActions';
import { JobIdCell } from './JobIdCell';
import {
  calculateDuration,
  formatDuration,
  getRelativeTime,
} from '@/modules/shared/utils/dateUtils';
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
        <span className='text-xs font-semibold uppercase tracking-wider'>
          <Trans>Status</Trans>
        </span>
      ),
      cell: ({ row }) => <JobStatus job={row.original} />,
      size: 180,
    },
    {
      accessorKey: 'jobId',
      header: () => (
        <span className='text-xs font-semibold uppercase tracking-wider'>
          <Trans>Job ID</Trans>
        </span>
      ),
      cell: ({ row }) => <JobIdCell job={row.original} />,
      size: 200,
    },
    {
      id: 'timeline',
      header: () => (
        <span className='text-xs font-semibold uppercase tracking-wider'>
          <Trans>Timeline</Trans>
        </span>
      ),
      cell: ({ row }) => <JobTimeline nodeExecutions={row.original.nodeExecutions} />,
      size: 300,
    },
    {
      id: 'duration',
      header: () => (
        <span className='text-xs font-semibold uppercase tracking-wider'>
          <Trans>Duration</Trans>
        </span>
      ),
      cell: ({ row }) => {
        const duration = calculateDuration(row.original.createdAt, row.original.updatedAt);
        return <span className='text-sm font-medium'>{formatDuration(duration)}</span>;
      },
      size: 120,
    },
    {
      accessorKey: 'createdAt',
      header: () => (
        <span className='text-xs font-semibold uppercase tracking-wider'>
          <Trans>Created</Trans>
        </span>
      ),
      cell: ({ row }) => <span className='text-sm'>{getRelativeTime(row.original.createdAt)}</span>,
      size: 120,
    },
    {
      id: 'actions',
      header: () => (
        <span className='text-xs font-semibold uppercase tracking-wider'>
          <Trans>Actions</Trans>
        </span>
      ),
      cell: ({ row }) => (
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
      ),
      size: 120,
    },
  ];
}
