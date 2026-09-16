import { useParams } from 'react-router-dom';
import { Trans, useLingui } from '@lingui/react/macro';
import { Skeleton } from '@shadcn/skeleton.tsx';
import { ErrorState } from '@shared/presentation/ui/feedback/ErrorState.tsx';
import { useResourceError } from '@shared/presentation/hooks/use-resource-error.ts';
import { useJob } from '@/modules/features/jobs/presentation/hooks/use-job.ts';
import { useOpenLogPanels } from '@/modules/features/jobs/presentation/hooks/use-open-log-panels.ts';
import { JobSummary } from '@/modules/features/jobs/presentation/ui/job-details/JobSummary.tsx';
import { JobNodeLogs } from '@/modules/features/jobs/presentation/ui/job-details/JobNodeLogs.tsx';

/**
 * One job: what it did, and what it printed.
 *
 * Which log panels are open lives in the URL rather than in state, so a link can
 * open the page already showing one node's logs — which is what the timeline
 * segments on the jobs list and the pipeline dashboard link to.
 *
 * The page fills the viewport instead of growing with its content: the logs are
 * what the page is for, so they take the room the summary leaves and scroll
 * inside it, rather than pushing the whole page into a scroll of its own.
 */
export const JobDetailsPage = () => {
  const { t } = useLingui();
  const { jobId } = useParams<{ jobId: string }>();
  const { job, isLoading, isError, error } = useJob(jobId ?? '');
  const { openNodeIds, isWholeJobOpen, toggleNode, openPanel, showWholeJob } = useOpenLogPanels(
    job?.nodeExecutions.map((node, index) => node.id || String(index)) ?? [],
  );

  const { redirecting } = useResourceError({
    error,
    redirectTo: '..',
    notFoundMessage: t`Job not found`,
  });

  if (!jobId) {
    return <ErrorState message={<Trans>Job ID is missing</Trans>} />;
  }

  if (redirecting) return null;
  if (isLoading) return <Skeleton className='h-72 w-full rounded-xl' />;
  if (isError || !job) return <ErrorState message={<Trans>Unable to load this job</Trans>} />;

  return (
    <div className='flex h-full min-h-0 w-full flex-col gap-6'>
      <JobSummary job={job} onOpenNode={openPanel} />
      <JobNodeLogs
        job={job}
        openNodeIds={openNodeIds}
        isWholeJobOpen={isWholeJobOpen}
        onToggleNode={toggleNode}
        onShowWholeJob={showWholeJob}
      />
    </div>
  );
};
