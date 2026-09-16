import { useParams, useSearchParams } from 'react-router-dom';
import { Trans, useLingui } from '@lingui/react/macro';
import { Skeleton } from '@shadcn/skeleton.tsx';
import { ErrorState } from '@shared/presentation/ui/feedback/ErrorState.tsx';
import { useResourceError } from '@shared/presentation/hooks/use-resource-error.ts';
import { useJob } from '@/modules/features/jobs/presentation/hooks/use-job.ts';
import { JobSummary } from '@/modules/features/jobs/presentation/ui/job-details/JobSummary.tsx';
import { JobNodeLogs } from '@/modules/features/jobs/presentation/ui/job-details/JobNodeLogs.tsx';

/**
 * One job: what it did, and what it printed.
 *
 * The selected node lives in the URL rather than in state, so a link can open
 * the page already scoped to one node's logs — which is what the timeline
 * segments on the jobs list and the pipeline dashboard link to.
 */
export const JobDetailsPage = () => {
  const { t } = useLingui();
  const { pipelineId, jobId } = useParams<{ pipelineId: string; jobId: string }>();
  const [searchParams, setSearchParams] = useSearchParams();
  const { job, isLoading, isError, error } = useJob(jobId ?? '');

  const { redirecting } = useResourceError({
    error,
    redirectTo: '..',
    notFoundMessage: t`Job not found`,
  });

  const selectNode = (nodeId?: string) => {
    const next = new URLSearchParams(searchParams);
    if (nodeId) next.set('nodeId', nodeId);
    else next.delete('nodeId');
    setSearchParams(next, { replace: true });
  };

  if (!pipelineId || !jobId) {
    return <ErrorState message={<Trans>Job ID is missing</Trans>} />;
  }

  if (redirecting) return null;
  if (isLoading) return <Skeleton className='h-72 w-full rounded-xl' />;
  if (isError || !job) return <ErrorState message={<Trans>Unable to load this job</Trans>} />;

  return (
    <div className='flex w-full min-h-full flex-col gap-6 pb-8'>
      <JobSummary job={job} pipelineId={pipelineId} onSelectNode={selectNode} />
      <JobNodeLogs
        job={job}
        selectedNodeId={searchParams.get('nodeId') ?? undefined}
        onSelectNode={selectNode}
      />
    </div>
  );
};
