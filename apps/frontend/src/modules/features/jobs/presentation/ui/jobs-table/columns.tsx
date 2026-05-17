import type { ColumnDef } from '@tanstack/react-table';
import type { Job } from '@/modules/features/jobs/domain/models/job.model.ts';
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
  onOpenJobLog: (jobId: string) => void;
};

export function createJobColumns(meta: JobColumnMeta): ColumnDef<Job>[] {
  return [
    {
      accessorKey: 'status',
      header: () => <Trans>Status</Trans>,
      cell: ({ row }) => <JobStatus job={row.original} />,
      size: 160,
      minSize: 140,
    },
    {
      accessorKey: 'id',
      header: () => <Trans>Job ID</Trans>,
      cell: ({ row }) => <JobIdCell job={row.original} />,
      size: 180,
      minSize: 150,
    },
    {
      id: 'timeline',
      header: () => <Trans>Timeline</Trans>,
      cell: ({ row }) => <JobTimeline nodeExecutions={row.original.nodeExecutions} />,
      size: undefined,
      minSize: 200,
    },
    {
      id: 'duration',
      header: () => <Trans>Duration</Trans>,
      cell: ({ row }) => {
        const duration = calculateDuration(row.original.createdAt, row.original.updatedAt);
        return (
          <span className='text-sm font-medium whitespace-nowrap'>{formatDuration(duration)}</span>
        );
      },
      size: 100,
      minSize: 80,
    },
    {
      accessorKey: 'createdAt',
      header: () => <Trans>Created</Trans>,
      cell: ({ row }) => (
        <span className='text-sm whitespace-nowrap'>{getRelativeTime(row.original.createdAt)}</span>
      ),
      size: 100,
      minSize: 80,
    },
    {
      id: 'actions',
      header: () => <Trans>Actions</Trans>,
      cell: ({ row }) => (
        <JobActions
          onView={e => {
            e.stopPropagation();
            meta.onView(row.original.id);
          }}
          onDelete={e => {
            e.stopPropagation();
            meta.onDelete(row.original.id);
          }}
          onOpenJobLog={e => {
            e.stopPropagation();
            meta.onOpenJobLog(row.original.id);
          }}
        />
      ),
      size: 100,
      minSize: 80,
    },
  ];
}
