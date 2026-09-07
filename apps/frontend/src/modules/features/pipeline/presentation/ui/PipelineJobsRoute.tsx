import { useParams } from 'react-router-dom';
import { JobsPage } from '@/modules/features/jobs/presentation/ui/Jobs.page.tsx';
import { useRunPipeline } from '@/modules/features/pipeline/presentation/hooks/use-run-pipeline.ts';

/**
 * The jobs page as mounted under a pipeline.
 *
 * `jobs` renders the list; running the pipeline is a pipeline operation, so the
 * composition happens here — on the side that is already allowed to depend on
 * `jobs`. This keeps the dependency one-way (pipeline → jobs) instead of the
 * mutual import the "Run" button used to require.
 */
export const PipelineJobsRoute = () => {
  const { pipelineId } = useParams<{ pipelineId: string }>();
  const runPipeline = useRunPipeline();

  const handleRun = async () => {
    if (!pipelineId) return;
    try {
      await runPipeline.mutateAsync(pipelineId);
    } catch {
      // Toast shown by the global MutationCache onError handler.
    }
  };

  return <JobsPage onRun={handleRun} />;
};
