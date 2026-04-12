import type { JobResponse } from '@/generated/job.ts';
import { ListCard, type ListCardSection } from '@shared/presentation/ui/ListCard.tsx';
import type { SyntheticEvent } from 'react';
import { JobActions } from './JobActions.tsx';
import { useDeleteJob } from '@/modules/features/jobs/presentation/hooks/useDeleteJob.ts';
import { useJobsStore } from '@/modules/features/jobs/presentation/stores/useJobsStore.ts';
import { useState } from 'react';
import { Copy, Check } from 'lucide-react';
import { JobNodesList } from './JobNodesList.tsx';
import { Button } from '@/modules/shared/presentation/ui/shadcn';
import { ConfirmOperationAlertDialog } from '@shared/presentation/ui/ConfirmOperationAlertDialog.tsx';
import { JobTimeline } from '@/modules/features/jobs/presentation/ui/jobs-table/JobTimeline.tsx';
import { getColumnConfig } from '@/modules/features/jobs/presentation/config/jobsTableConfig.ts';
import { JobStatus } from '@/modules/features/jobs/presentation/ui/jobs-table/JobStatus.tsx';

export type JobRowProps = {
  job: JobResponse;
  pipelineId: string;
  onClick?: () => void;
  selected?: boolean;
};

const calculateDuration = (createdAt: string, updatedAt: string): string => {
  const start = new Date(createdAt).getTime();
  const end = new Date(updatedAt).getTime();
  const durationMs = end - start;
  const seconds = Math.floor(durationMs / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);

  if (hours > 0) return `${hours}h ${minutes % 60}m`;
  if (minutes > 0) return `${minutes}m ${seconds % 60}s`;
  return `${seconds}s`;
};

const getRelativeTime = (dateString: string): string => {
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSeconds = Math.floor(diffMs / 1000);
  const diffMinutes = Math.floor(diffSeconds / 60);
  const diffHours = Math.floor(diffMinutes / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffDays > 0) return `${diffDays}d ago`;
  if (diffHours > 0) return `${diffHours}h ago`;
  if (diffMinutes > 0) return `${diffMinutes}m ago`;
  return `${diffSeconds}s ago`;
};

/**
 * Component representing a single job row in the jobs table
 */
export const JobRow = ({ job, pipelineId, onClick, selected }: JobRowProps) => {
  const deleteJob = useDeleteJob(pipelineId);
  const expandedJobId = useJobsStore(state => state.expandedJobId);
  const toggleExpand = useJobsStore(state => state.toggleExpand);
  const [copied, setCopied] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);

  const isExpanded = expandedJobId === job.jobId;

  const handleCopyJobId = (e: SyntheticEvent) => {
    e.stopPropagation();
    navigator.clipboard.writeText(job.jobId);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleView = (e: SyntheticEvent) => {
    e.stopPropagation();
    toggleExpand(isExpanded ? null : job.jobId);
  };

  const handleDelete = async () => {
    try {
      await deleteJob.mutateAsync(job.jobId);
      setDeleteDialogOpen(false);
    } catch (error) {
      console.error('Error deleting job:', error);
    }
  };

  const handleDeleteClick = (e: SyntheticEvent) => {
    e.stopPropagation();
    setDeleteDialogOpen(true);
  };

  const statusConfig = getColumnConfig('status');
  const jobIdConfig = getColumnConfig('jobId');
  const timelineConfig = getColumnConfig('timeline');
  const durationConfig = getColumnConfig('duration');
  const createdConfig = getColumnConfig('created');
  const actionsConfig = getColumnConfig('actions');

  const sections: ListCardSection[] = [
    // STATUS
    {
      width: statusConfig.width,
      className: statusConfig.className,
      content: <JobStatus job={job} />,
    },
    // JOB ID
    {
      width: jobIdConfig.width,
      className: jobIdConfig.className,
      content: (
        <div className='flex items-center gap-2'>
          <span className='font-mono text-sm truncate'>{job.jobId.slice(0, 12)}...</span>
          <Button
            size='icon'
            variant='ghost'
            className='h-6 w-6 shrink-0'
            onClick={handleCopyJobId}
          >
            {copied ? <Check className='w-3 h-3 text-green-500' /> : <Copy className='w-3 h-3' />}
          </Button>
        </div>
      ),
    },
    // TIMELINE
    {
      width: timelineConfig.width,
      className: timelineConfig.className,
      content: <JobTimeline nodeExecutions={job.nodeExecutions} />,
    },
    // DURATION
    {
      width: durationConfig.width,
      className: durationConfig.className,
      content: (
        <span className='text-sm font-medium'>
          {calculateDuration(job.createdAt, job.updatedAt)}
        </span>
      ),
    },
    // CREATED
    {
      width: createdConfig.width,
      className: createdConfig.className,
      content: <span className='text-sm'>{getRelativeTime(job.createdAt)}</span>,
    },
    // ACTIONS
    {
      width: actionsConfig.width,
      className: actionsConfig.className,
      noSeparator: actionsConfig.noSeparator,
      content: <JobActions onView={handleView} onDelete={handleDeleteClick} />,
    },
  ];

  return (
    <div>
      <ListCard sections={sections} onClick={onClick} selected={selected} />
      <JobNodesList nodeExecutions={job.nodeExecutions} isExpanded={isExpanded} />
      <ConfirmOperationAlertDialog
        open={deleteDialogOpen}
        onOpenChange={setDeleteDialogOpen}
        onContinue={handleDelete}
        title='Delete Job'
        description={`Are you sure you want to delete job ${job.jobId.slice(0, 12)}...? This action cannot be undone.`}
      />
    </div>
  );
};
