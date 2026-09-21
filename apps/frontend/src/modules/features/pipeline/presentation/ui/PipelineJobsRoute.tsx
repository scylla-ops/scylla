import { useCallback } from 'react';
import { useParams } from 'react-router-dom';
import { loadJobsPage } from '@/modules/features/jobs';
import { LazySvelteIsland } from '@shared/presentation/ui/svelte/LazySvelteIsland.tsx';
import { useRunPipeline } from '@/modules/features/pipeline/presentation/hooks/use-run-pipeline.ts';

/**
 * The jobs page as mounted under a pipeline.
 *
 * `jobs` renders the list; running the pipeline is a pipeline operation, so the
 * composition happens here — on the side that is already allowed to depend on
 * `jobs`. This keeps the dependency one-way (pipeline → jobs) instead of the
 * mutual import the "Run" button used to require.
 *
 * The page is Svelte as of Phase 3 and this module is not, so it is mounted as
 * an island. `onRun` crosses that boundary unchanged — a callback is just a
 * value — which is what makes this composition survive the migration; a
 * *component* passed as a prop would not have (see `refacto_svelte.md` §2).
 * The island goes away when `pipeline` is migrated in Phase 5.
 */
export const PipelineJobsRoute = () => {
  const { pipelineId } = useParams<{ pipelineId: string }>();
  const runPipeline = useRunPipeline();

  // Stable: `LazySvelteIsland` re-fetches the chunk whenever `load` changes
  // identity, and an inline arrow is a new function on every render.
  const load = useCallback(() => loadJobsPage(), []);

  const handleRun = async () => {
    if (!pipelineId) return;
    try {
      await runPipeline.mutateAsync(pipelineId);
    } catch {
      // Toast shown by the global MutationCache onError handler.
    }
  };

  return <LazySvelteIsland load={load} props={{ pipelineId, onRun: handleRun }} />;
};
