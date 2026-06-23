import { DataTable } from '@shared/presentation/ui/data-display/DataTable.tsx';
import { createPipelineColumns } from './columns.tsx';
import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';
import { useRunPipeline } from '../../../hooks/use-run-pipeline.ts';
import { useDuplicatePipeline } from '../../../hooks/use-duplicate-pipeline.ts';
import { useSelection } from '@shared/presentation/hooks/use-selection.ts';
import { useState } from 'react';
import type { PipelineMetadata } from '@/modules/features/pipeline/domain/structs/pipeline.struct.ts';
import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';

type PipelineTableProps = {
  pipelines: PipelineMetadata[];
  jobsByPipelineId: Map<string, JobEntity[]>;
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
  const { goToEditPipeline, goToJobs, goToTriggers } = useScyllaNavigate();
  const { mutateAsync } = useRunPipeline();
  const duplicatePipeline = useDuplicatePipeline();
  const [runningPipelines, setRunningPipelines] = useState<Set<string>>(new Set());

  const columns = createPipelineColumns({
    onRun: pipelineId => {
      setRunningPipelines(prev => new Set(prev).add(pipelineId));
      mutateAsync(pipelineId)
        .finally(() => {
          setRunningPipelines(prev => {
            const newSet = new Set(prev);
            newSet.delete(pipelineId);
            return newSet;
          });
        })
        .catch(() => {
          // Toast shown by the global MutationCache onError handler.
        });
    },
    onEdit: pipeline => {
      goToEditPipeline(pipeline.id, pipeline.name);
    },
    onDuplicate: pipeline => {
      duplicatePipeline.mutate(pipeline.id);
    },
    onViewJobs: pipeline => {
      goToJobs(pipeline);
    },
    onViewTriggers: pipeline => {
      goToTriggers(pipeline);
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
