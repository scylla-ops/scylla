import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';
import { useJobsStore } from '@/modules/features/jobs/presentation/stores/use-jobs.store.ts';
import { DataTable } from '@/modules/shared/presentation/ui/data-display/DataTable';
import { createJobColumns } from './columns';
import { useState } from 'react';
import { ConfirmOperationAlertDialog } from '@shared/presentation/ui/feedback/ConfirmOperationAlertDialog.tsx';
import { useDeleteJobs } from '@/modules/features/jobs/presentation/hooks/use-delete-jobs.ts';
import { JobNodesList } from './JobNodesList';
import { JobLogDialog } from '@/modules/features/jobs/presentation/ui/jobs-table/jobs-log/JobLogDialog.tsx';
import { useSelection } from '@shared/presentation/hooks/use-selection.ts';
import { useLingui } from '@lingui/react/macro';

type JobsTableProps = {
  jobs: JobEntity[];
  pipelineId: string;
};

export const JobsTable = ({ jobs, pipelineId }: JobsTableProps) => {
  const { t } = useLingui();
  const expandedJobId = useJobsStore(state => state.expandedJobId);
  const toggleExpand = useJobsStore(state => state.toggleExpand);
  const { selectedIds, select } = useSelection('jobs');

  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [jobToDelete, setJobToDelete] = useState<string | null>(null);

  const deleteJob = useDeleteJobs(pipelineId);

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
