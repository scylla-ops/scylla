import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';
import { useJobsStore } from '@/modules/features/jobs/presentation/stores/use-jobs.store.ts';
import { DataTable } from '@/modules/shared/presentation/ui/data-display/DataTable';
import { createJobColumns } from './columns';
import { useState } from 'react';
import { ConfirmOperationAlertDialog } from '@shared/presentation/ui/feedback/ConfirmOperationAlertDialog.tsx';
import { useDeleteJobs } from '@/modules/features/jobs/presentation/hooks/use-delete-jobs.ts';
import { JobNodesList } from './JobNodesList';
import { JobLogDialog } from '@/modules/features/jobs/presentation/ui/jobs-table/jobs-log/JobLogDialog.tsx';
import { useFeatureSelection } from '@shared/presentation/hooks/use-feature-selection.ts';
import { Trans, useLingui } from '@lingui/react/macro';
import { Permission, useCan } from '@platform/authz';
import { getStatusConfig } from '@shared/utils/status-config.ts';

type JobsTableProps = {
  jobs: JobEntity[];
  pipelineId: string;
};

export const JobsTable = ({ jobs, pipelineId }: JobsTableProps) => {
  const { t, i18n } = useLingui();
  const expandedJobId = useJobsStore(state => state.expandedJobId);
  const toggleExpand = useJobsStore(state => state.toggleExpand);
  const deleteJob = useDeleteJobs(pipelineId);
  const { selectedIds, select, headerProps } = useFeatureSelection(
    'jobs',
    jobs.map(job => job.id),
    { deleteItem: id => deleteJob.mutateAsync(id) },
  );
  const canDelete = useCan(Permission.DELETE_JOB);

  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [jobToDelete, setJobToDelete] = useState<string | null>(null);

  const [logJobId, setLogJobId] = useState<string | undefined>(undefined);
  const [logNodeId, setLogNodeId] = useState<string | undefined>(undefined);

  const handleDelete = async () => {
    if (!jobToDelete) return;
    try {
      await deleteJob.mutateAsync(jobToDelete);
    } finally {
      setDeleteDialogOpen(false);
      setJobToDelete(null);
    }
  };

  // Only the statuses actually present are offered, each with its shared label.
  const statusOptions = Array.from(new Set(jobs.map(job => job.status)))
    .sort()
    .map(status => ({ value: status, label: i18n._(getStatusConfig(status).label) }));

  const columns = createJobColumns({
    pipelineId,
    onDelete: jobId => {
      setJobToDelete(jobId);
      setDeleteDialogOpen(true);
    },
    onView: jobId => {
      toggleExpand(expandedJobId === jobId ? null : jobId);
    },
    onOpenJobLog: jobId => {
      setLogJobId(jobId);
    },
  });

  return (
    <>
      <DataTable
        columns={columns}
        data={jobs}
        onRowClick={row => select(row.original.id)}
        getRowId={(row, index) => row.id || index.toString()}
        isRowSelected={row => selectedIds.includes(row.id)}
        searchable
        searchPlaceholder={t`Search a job…`}
        searchValues={job => [job.id, job.status]}
        facets={[
          { id: 'status', label: t`Status`, value: job => job.status, options: statusOptions },
        ]}
        selection={{
          ...headerProps,
          canDelete,
          deleteDeniedReason: <Trans>You don't have permission to delete jobs.</Trans>,
        }}
        isRowExpanded={row => expandedJobId === row.id}
        expandedContent={row => (
          <JobNodesList
            jobId={row.original.id}
            nodeExecutions={row.original.nodeExecutions}
            isExpanded={true}
            onCollapse={() => toggleExpand(null)}
          />
        )}
        alignColumnsCenter
        alignRowsCenter
        tableLayoutFixed
      />
      <ConfirmOperationAlertDialog
        open={deleteDialogOpen}
        onOpenChange={setDeleteDialogOpen}
        onContinue={handleDelete}
        title={t`Delete Job`}
        description={`Are you sure you want to delete job ${jobToDelete}? This action cannot be undone.`}
      />

      <JobLogDialog
        jobId={logJobId}
        nodeId={logNodeId}
        onClose={() => {
          setLogJobId(undefined);
          setLogNodeId(undefined);
        }}
      />
    </>
  );
};
