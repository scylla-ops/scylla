import type { ColumnDef } from '@tanstack/react-table';
import { PipelineStatus } from './PipelineStatus.tsx';
import { PipelineLastJob } from './PipelineLastJob.tsx';
import { PipelineActions } from './PipelineActions.tsx';
import { PipelineChart } from '../PipelineChart.tsx';
import { Trans } from '@lingui/react/macro';
import type { PipelineMetadata } from '@/modules/features/pipeline/domain/structs/pipeline.struct.ts';
import { toStatusState } from '@shared/utils/job-status.utils.ts';
import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';
type PipelineColumnMeta = {
  onRun: (pipelineId: string) => void;
  onEdit: (pipeline: PipelineMetadata) => void;
  onDuplicate: (pipeline: PipelineMetadata) => void;
  onViewJobs: (pipeline: PipelineMetadata) => void;
  onViewTriggers: (pipeline: PipelineMetadata) => void;

  runningPipelines: Set<string>;
  duplicatingPipelineId: string | undefined;

  jobsByPipelineId: Map<string, JobEntity[]>;
  isJobsLoading?: boolean;
  isJobsError?: boolean;
  canListJobs?: boolean;
};

export const createPipelineColumns = (meta: PipelineColumnMeta): ColumnDef<PipelineMetadata>[] => [
  {
    id: 'status',
    header: () => <Trans>Status</Trans>,
    cell: ({ row }) => {
      const lastJob = meta.jobsByPipelineId.get(row.original.id)?.[0];
      return <PipelineStatus status={toStatusState(lastJob?.status)} pipeline={row.original} />;
    },
    size: 350,
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
          isForbidden={meta.canListJobs === false}
          maxJobs={10}
        />
      );
    },
    size: 400,
  },
  {
    id: 'metadata',
    header: () => <Trans>Last Run</Trans>,
    cell: ({ row }) => {
      const jobs = meta.jobsByPipelineId.get(row.original.id) ?? [];
      return <PipelineLastJob jobs={jobs} />;
    },
    size: 200,
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
        onDuplicate={e => {
          e.stopPropagation();
          meta.onDuplicate(row.original);
        }}
        onViewJobs={e => {
          e.stopPropagation();
          meta.onViewJobs(row.original);
        }}
        onViewTriggers={e => {
          e.stopPropagation();
          meta.onViewTriggers(row.original);
        }}
        isRunning={meta.runningPipelines.has(row.original.id)}
        isDuplicating={meta.duplicatingPipelineId === row.original.id}
      />
    ),
    size: 200,
  },
];
