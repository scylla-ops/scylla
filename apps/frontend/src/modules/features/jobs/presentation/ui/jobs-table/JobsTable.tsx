import type { JobResponse } from '@/generated/job.ts';
import { useJobsStore } from '@/modules/features/jobs/presentation/stores/use-jobs.store.ts';
import { DataTable } from '@/modules/shared/presentation/ui/DataTable';
import { createJobColumns } from './columns';
import { useState } from 'react';
import { ConfirmOperationAlertDialog } from '@shared/presentation/ui/ConfirmOperationAlertDialog.tsx';
import { useDeleteJobs } from '@/modules/features/jobs/presentation/hooks/use-delete-jobs.ts';
import { JobNodesList } from './JobNodesList';

type JobsTableProps = {
  jobs: JobResponse[];
  pipelineId: string;
};

export const JobsTable = ({ jobs, pipelineId }: JobsTableProps) => {
  const selectJob = useJobsStore(state => state.selectJob);
  const selectedJobIds = useJobsStore(state => state.selectedJobIds);
  const expandedJobId = useJobsStore(state => state.expandedJobId);
  const toggleExpand = useJobsStore(state => state.toggleExpand);

  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [jobToDelete, setJobToDelete] = useState<string | null>(null);

  const deleteJob = useDeleteJobs(pipelineId);

  const handleDelete = async () => {
    if (!jobToDelete) return;
    try {
      await deleteJob.mutateAsync(jobToDelete);
      setDeleteDialogOpen(false);
      setJobToDelete(null);
    } catch (error) {
      console.error('Error deleting job:', error);
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
  });

  return (
    <>
      <DataTable
        columns={columns}
        data={jobs}
        alignCenter
        onRowClick={row => selectJob(row.original.jobId)}
        getRowId={(row, index) => row.jobId || index.toString()}
        isRowSelected={row => selectedJobIds.includes(row.jobId)}
        isRowExpanded={row => expandedJobId === row.jobId}
        expandedContent={row => (
          <JobNodesList nodeExecutions={row.original.nodeExecutions} isExpanded={true} />
        )}
      />
      <ConfirmOperationAlertDialog
        open={deleteDialogOpen}
        onOpenChange={setDeleteDialogOpen}
        onContinue={handleDelete}
        title='Delete Job'
        description={`Are you sure you want to delete job ${jobToDelete}? This action cannot be undone.`}
      />
    </>
  );
};
