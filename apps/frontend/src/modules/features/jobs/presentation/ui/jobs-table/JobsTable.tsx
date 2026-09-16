import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';
import { DataTable } from '@/modules/shared/presentation/ui/data-display/DataTable';
import { createJobColumns } from './columns';
import { useState } from 'react';
import { ConfirmOperationAlertDialog } from '@shared/presentation/ui/feedback/ConfirmOperationAlertDialog.tsx';
import { useDeleteJobs } from '@/modules/features/jobs/presentation/hooks/use-delete-jobs.ts';
import { useSelection } from '@shared/presentation/hooks/use-selection.ts';
import { useScyllaNavigate } from '@platform/context';
import { useLingui } from '@lingui/react/macro';

type JobsTableProps = {
  jobs: JobEntity[];
  pipelineId: string;
};

export const JobsTable = ({ jobs, pipelineId }: JobsTableProps) => {
  const { t } = useLingui();
  const { selectedIds, select } = useSelection('jobs');
  const { goToJobDetails } = useScyllaNavigate();

  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [jobToDelete, setJobToDelete] = useState<string | null>(null);

  const deleteJob = useDeleteJobs(pipelineId);

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
    onView: (jobId, nodeId) => {
      goToJobDetails(pipelineId, jobId, { nodeId });
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
        alignColumnsCenter
        alignRowsCenter
      />
      <ConfirmOperationAlertDialog
        open={deleteDialogOpen}
        onOpenChange={setDeleteDialogOpen}
        onContinue={handleDelete}
        title={t`Delete Job`}
        description={`Are you sure you want to delete job ${jobToDelete}? This action cannot be undone.`}
      />
    </>
  );
};
