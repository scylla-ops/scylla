import type { PipelineSummary } from '@/generated/pipeline.ts';
import { usePipelineDashboardStore } from '@/modules/features/pipeline-dashboard/presentation/stores/usePipelineDashboardStore.ts';
import { DataTable } from '@/modules/shared/presentation/ui/DataTable';
import { createPipelineColumns } from './columns';
import { useScyllaNavigate } from '@shared/presentation/hooks/useScyllaNavigate.ts';
import { useRunPipeline } from '../../hooks/useRunPipeline';

type PipelineTableProps = {
  pipelines: PipelineSummary[];
};

export const PipelineTable = ({ pipelines }: PipelineTableProps) => {
  const selectPipeline = usePipelineDashboardStore(state => state.selectPipeline);
  const selectedPipelineIds = usePipelineDashboardStore(state => state.selectedPipelineIds);
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
      onRowClick={row => selectPipeline(row.original.pipelineId)}
      getRowId={(row, index) => row.pipelineId || index.toString()}
      isRowSelected={row => selectedPipelineIds.includes(row.pipelineId)}
    />
  );
};
