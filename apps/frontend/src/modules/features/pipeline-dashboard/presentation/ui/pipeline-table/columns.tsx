import type { ColumnDef } from '@tanstack/react-table';
import type { PipelineSummary } from '@/generated/pipeline.ts';
import type { JobResponse } from '@/generated/job.ts';
import { PipelineStatus } from './PipelineStatus';
import { PipelineMetadata } from './PipelineMetadata';
import { PipelineActions } from './PipelineActions';
import { PipelineChart } from '../PipelineChart';
import { Trans } from '@lingui/react/macro';

type PipelineColumnMeta = {
  onRun: (pipelineId: string) => void;
  onEdit: (pipeline: PipelineSummary) => void;
  onViewJobs: (pipelineId: string) => void;

  runningPipelines: Set<string>;

  jobsByPipelineId: Map<string, JobResponse[]>;
  isJobsLoading?: boolean;
  isJobsError?: boolean;
};

export const createPipelineColumns = (meta: PipelineColumnMeta): ColumnDef<PipelineSummary>[] => [
  {
    id: 'status',
    header: () => (
      <div className='w-full text-center text-xs font-semibold uppercase tracking-wider'>
        <Trans>Status</Trans>
      </div>
    ),
    cell: ({ row }) => {
      const lastJob = meta.jobsByPipelineId.get(row.original.pipelineId)?.[0];
      console.log(lastJob?.status);
      const status = lastJob?.status as 'idle' | 'running' | 'success' | 'failed' | undefined;
      return <PipelineStatus status={status} pipeline={row.original} />;
    },
    size: 240,
  },
  {
    id: 'history',
    header: () => (
      <div className='w-full text-center text-xs font-semibold uppercase tracking-wider'>
        <Trans>History</Trans>
      </div>
    ),
    cell: ({ row }) => {
      const jobs = meta.jobsByPipelineId.get(row.original.pipelineId) ?? [];
      return (
        <div className='w-full'>
          <PipelineChart
            jobs={jobs}
            isLoading={meta.isJobsLoading}
            isError={meta.isJobsError}
            maxJobs={10}
          />
        </div>
      );
    },
    size: 300,
  },
  {
    id: 'metadata',
    header: () => (
      <div className='w-full text-center text-xs font-semibold uppercase tracking-wider'>
        <Trans>Last Run</Trans>
      </div>
    ),
    cell: ({ row }) => {
      const jobs = meta.jobsByPipelineId.get(row.original.pipelineId) ?? [];
      return (
        <div className='flex w-full flex-col gap-1'>
          <PipelineMetadata jobs={jobs} />
        </div>
      );
    },
    size: 140,
  },
  {
    id: 'actions',
    header: () => (
      <div className='text-center w-full text-xs font-semibold uppercase tracking-wider'>
        <Trans>Actions</Trans>
      </div>
    ),
    cell: ({ row }) => (
      <PipelineActions
        onRun={e => {
          e.stopPropagation();
          meta.onRun(row.original.pipelineId);
        }}
        onEdit={e => {
          e.stopPropagation();
          meta.onEdit(row.original);
        }}
        onViewJobs={e => {
          e.stopPropagation();
          meta.onViewJobs(row.original.pipelineId);
        }}
        isRunning={meta.runningPipelines.has(row.original.pipelineId)}
      />
    ),
    size: 140,
  },
];
