import { DataTable } from '@shared/presentation/ui/data-display/DataTable.tsx';
import { createPipelineColumns } from './columns.tsx';
import { useScyllaNavigate } from '@platform/context';
import { useRunPipeline } from '../../../hooks/use-run-pipeline.ts';
import { useDuplicatePipeline } from '../../../hooks/use-duplicate-pipeline.ts';
import { useFeatureSelection } from '@shared/presentation/hooks/use-feature-selection.ts';
import { useDeletePipeline } from '../../../hooks/use-delete-pipeline.ts';
import { Permission, useCan } from '@platform/authz';
import { Trans, useLingui } from '@lingui/react/macro';
import { getStatusConfig } from '@shared/utils/status-config.ts';
import { useState } from 'react';
import type { PipelineMetadata } from '@/modules/features/pipeline/domain/structs/pipeline.struct.ts';
import type { JobEntity } from '@/modules/features/jobs';

/** Facet value for a pipeline that has never produced a job. */
const NEVER_RUN = 'never-run';

type PipelineTableProps = {
  pipelines: PipelineMetadata[];
  jobsByPipelineId: Map<string, JobEntity[]>;
  isJobsLoading?: boolean;
  isJobsError?: boolean;
  canListJobs?: boolean;
};

export const PipelineTable = ({
  pipelines,
  jobsByPipelineId,
  isJobsLoading,
  isJobsError,
  canListJobs,
}: PipelineTableProps) => {
  const deletePipeline = useDeletePipeline();
  const { selectedIds, select, headerProps } = useFeatureSelection(
    'pipelines',
    pipelines.map(pipeline => pipeline.id),
    { deleteItem: id => deletePipeline.mutateAsync(id) },
  );
  const canDelete = useCan(Permission.DELETE_PIPELINE);
  const { t, i18n } = useLingui();
  const { goToEditPipeline, goToJobs, goToTriggers } = useScyllaNavigate();
  const { mutateAsync } = useRunPipeline();
  const duplicatePipeline = useDuplicatePipeline();
  const [runningPipelines, setRunningPipelines] = useState<Set<string>>(new Set());

  // A pipeline's status is its last job's — the same value the Status column shows.
  const lastJobStatus = (pipelineId: string) =>
    jobsByPipelineId.get(pipelineId)?.[0]?.status ?? NEVER_RUN;

  const statusOptions = Array.from(new Set(pipelines.map(pipeline => lastJobStatus(pipeline.id))))
    .sort()
    .map(status => ({
      value: status,
      label: status === NEVER_RUN ? t`Never run` : i18n._(getStatusConfig(status).label),
    }));

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
      goToJobs(pipeline.id, pipeline.name);
    },
    onViewTriggers: pipeline => {
      goToTriggers(pipeline.id, pipeline.name);
    },
    runningPipelines: runningPipelines,
    duplicatingPipelineId: duplicatePipeline.isPending ? duplicatePipeline.variables : undefined,
    jobsByPipelineId,
    isJobsLoading: isJobsLoading,
    isJobsError: isJobsError,
    canListJobs: canListJobs,
  });

  return (
    <DataTable
      columns={columns}
      data={pipelines}
      onRowClick={row => select(row.original.id)}
      getRowId={(row, index) => row.id || index.toString()}
      isRowSelected={row => selectedIds.includes(row.id)}
      searchable
      searchPlaceholder={t`Search a pipeline…`}
      searchValues={pipeline => [pipeline.name, pipeline.id]}
      facets={[
        {
          id: 'status',
          label: t`Status`,
          value: pipeline => lastJobStatus(pipeline.id),
          options: statusOptions,
        },
      ]}
      selection={{
        ...headerProps,
        canDelete,
        deleteDeniedReason: <Trans>You don't have permission to delete pipelines.</Trans>,
      }}
      alignColumnsCenter
      alignRowsCenter
    />
  );
};
