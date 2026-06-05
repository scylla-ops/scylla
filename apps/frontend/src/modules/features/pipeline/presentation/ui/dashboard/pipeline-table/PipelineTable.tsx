import { DataTable } from '@shared/presentation/ui/DataTable.tsx';
import { createPipelineColumns } from './columns.tsx';
import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';
import { useRunPipeline } from '../../../hooks/use-run-pipeline.ts';
import { useSelection } from '@shared/presentation/hooks/use-selection.ts';
import { useState } from 'react';
import type { PipelineMetadata } from '@/modules/features/pipeline/domain/models/pipeline.model.ts';
import type { Job } from '@/modules/features/jobs/domain/models/job.model.ts';

type PipelineTableProps = {
  pipelines: PipelineMetadata[];
  jobsByPipelineId: Map<string, Job[]>;
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
      goToEditPipeline(pipeline.id, pipeline.name);
    },
    onViewJobs: pipeline => {
      goToJobs(pipeline);
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
      onRowClick={row => select(row.original.id)}
      getRowId={(row, index) => row.id || index.toString()}
      isRowSelected={row => selectedIds.includes(row.id)}
      alignColumnsCenter
      alignRowsCenter
    />
  );
};
