import type { ColumnDef } from '@tanstack/react-table';
import type { PipelineSummary } from '@/generated/pipeline.ts';
import { PipelineStatus } from './PipelineStatus';
import { PipelineMetadata } from './PipelineMetadata';
import { PipelineActions } from './PipelineActions';
import { PipelineChart } from '../PipelineChart';
import { Trans } from '@lingui/react/macro';

type PipelineColumnMeta = {
  onRun: (pipelineId: string) => void;
  onEdit: (pipeline: PipelineSummary) => void;
  onViewJobs: (pipelineId: string) => void;
};

export const createPipelineColumns = (meta: PipelineColumnMeta): ColumnDef<PipelineSummary>[] => [
  {
    id: 'status',
    header: () => (
      <div className='w-full text-center text-xs font-semibold uppercase tracking-wider'>
        <Trans>Status</Trans>
      </div>
    ),
    cell: ({ row }) => <PipelineStatus pipeline={row.original} />,
    size: 240,
  },
  {
    id: 'history',
    header: () => (
      <div className='w-full text-center text-xs font-semibold uppercase tracking-wider'>
        <Trans>History</Trans>
      </div>
    ),
    cell: ({ row }) => (
      <div className='w-full'>
        <PipelineChart pipelineId={row.original.pipelineId} />
      </div>
    ),
    size: 300,
  },
  {
    id: 'metadata',
    header: () => (
      <div className='text-center w-full text-xs font-semibold uppercase tracking-wider'>
        <Trans>Last Run</Trans>
      </div>
    ),
    cell: ({ row }) => (
      <div className='flex flex-col gap-1'>
        <PipelineMetadata pipelineId={row.original.pipelineId} />
      </div>
    ),
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
      />
    ),
    size: 140,
  },
];
