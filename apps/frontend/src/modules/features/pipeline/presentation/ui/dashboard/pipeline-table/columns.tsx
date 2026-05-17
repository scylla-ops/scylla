import type { ColumnDef } from '@tanstack/react-table';
import { PipelineStatus } from './PipelineStatus.tsx';
import { PipelineLastJob } from './PipelineLastJob.tsx';
import { PipelineActions } from './PipelineActions.tsx';
import { PipelineChart } from '../PipelineChart.tsx';
import { Trans } from '@lingui/react/macro';
import type { PipelineMetadata } from '@/modules/features/pipeline/domain/models/pipeline.model.ts';
import { toStatusState } from '@shared/utils/job-status.utils.ts';
import type { Job } from '@/modules/features/jobs/domain/models/job.model.ts';
type PipelineColumnMeta = {
  onRun: (pipelineId: string) => void;
  onEdit: (pipeline: PipelineMetadata) => void;
  onViewJobs: (pipeline: PipelineMetadata) => void;

  runningPipelines: Set<string>;

  jobsByPipelineId: Map<string, Job[]>;
  isJobsLoading?: boolean;
  isJobsError?: boolean;
};

export const createPipelineColumns = (meta: PipelineColumnMeta): ColumnDef<PipelineMetadata>[] => [
  {
    id: 'status',
    header: () => <Trans>Status</Trans>,
    cell: ({ row }) => {
      const lastJob = meta.jobsByPipelineId.get(row.original.id)?.[0];
      return <PipelineStatus status={toStatusState(lastJob?.status)} pipeline={row.original} />;
    },
    size: 240,
  },
  {
    id: 'history',
    header: () => <Trans>History</Trans>,
    cell: ({ row }) => {
      const jobs = meta.jobsByPipelineId.get(row.original.id) ?? [];
      return (
        <PipelineChart
          jobs={jobs}
          isLoading={meta.isJobsLoading}
          isError={meta.isJobsError}
          maxJobs={10}
        />
      );
    },
    size: 300,
  },
  {
    id: 'metadata',
    header: () => <Trans>Last Run</Trans>,
    cell: ({ row }) => {
      const jobs = meta.jobsByPipelineId.get(row.original.id) ?? [];
      return <PipelineLastJob jobs={jobs} />;
    },
    size: 140,
  },
  {
    id: 'actions',
    header: () => <Trans>Actions</Trans>,
    cell: ({ row }) => (
      <PipelineActions
        onRun={e => {
          e.stopPropagation();
          meta.onRun(row.original.id);
        }}
        onEdit={e => {
          e.stopPropagation();
          meta.onEdit(row.original);
        }}
        onViewJobs={e => {
          e.stopPropagation();
          meta.onViewJobs(row.original);
        }}
        isRunning={meta.runningPipelines.has(row.original.id)}
      />
    ),
    size: 140,
  },
];
