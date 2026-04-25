import type { PipelineSummary } from '@/generated/pipeline.ts';
import { DataTable } from '@/modules/shared/presentation/ui/DataTable';
import { createPipelineColumns } from './columns';
import { useScyllaNavigate } from '@shared/presentation/hooks/useScyllaNavigate.ts';
import { useRunPipeline } from '../../hooks/useRunPipeline';
import { useSelection } from '@shared/presentation/hooks/useSelection.ts';

type PipelineTableProps = {
  pipelines: PipelineSummary[];
};

export const PipelineTable = ({ pipelines }: PipelineTableProps) => {
  const { selectedIds, select } = useSelection('pipelines');
  const { goToEditPipeline, goToJobs } = useScyllaNavigate();
  const runPipeline = useRunPipeline();

  const columns = createPipelineColumns({
    onRun: pipelineId => {
      runPipeline.mutateAsync(pipelineId);
    },
    onEdit: pipeline => {
      goToEditPipeline(pipeline);
    },
    onViewJobs: pipelineId => {
      goToJobs(pipelineId);
    },
  });

  return (
    <DataTable
      columns={columns}
      data={pipelines}
      onRowClick={row => select(row.original.pipelineId)}
      getRowId={(row, index) => row.pipelineId || index.toString()}
      isRowSelected={row => selectedIds.includes(row.pipelineId)}
    />
  );
};
