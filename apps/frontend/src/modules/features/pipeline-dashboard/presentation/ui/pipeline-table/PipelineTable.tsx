import type { PipelineSummary } from '@/generated/pipeline.ts';
import type { JobResponse } from '@/generated/job.ts';
import { DataTable } from '@/modules/shared/presentation/ui/DataTable';
import { createPipelineColumns } from './columns';
import { useScyllaNavigate } from '@shared/presentation/hooks/useScyllaNavigate.ts';
import { useRunPipeline } from '../../hooks/useRunPipeline';
import { useSelection } from '@shared/presentation/hooks/useSelection.ts';
import { useState } from 'react';

type PipelineTableProps = {
  pipelines: PipelineSummary[];
  jobsByPipelineId: Map<string, JobResponse[]>;
  isJobsLoading?: boolean;
  isJobsError?: boolean;
};

export const PipelineTable = ({
  pipelines,
  jobsByPipelineId,
  isJobsLoading,
  isJobsError,
}: PipelineTableProps) => {
  const { selectedIds, select } = useSelection('pipelines');
  const { goToEditPipeline, goToJobs } = useScyllaNavigate();
  const { mutateAsync } = useRunPipeline();
  const [runningPipelines, setRunningPipelines] = useState<Set<string>>(new Set());

  const columns = createPipelineColumns({
    onRun: pipelineId => {
      setRunningPipelines(prev => new Set(prev).add(pipelineId));
      mutateAsync(pipelineId).finally(() => {
        setRunningPipelines(prev => {
          const newSet = new Set(prev);
          newSet.delete(pipelineId);
          return newSet;
        });
      });
    },
    onEdit: pipeline => {
      goToEditPipeline(pipeline);
    },
    onViewJobs: pipelineId => {
      goToJobs(pipelineId);
    },
    runningPipelines: runningPipelines,
    jobsByPipelineId,
    isJobsLoading: isJobsLoading,
    isJobsError: isJobsError,
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
