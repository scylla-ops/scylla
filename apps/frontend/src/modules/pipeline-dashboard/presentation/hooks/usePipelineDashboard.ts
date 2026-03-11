import { useCallback } from 'react';
import { usePipelineDashboardStore } from '@/modules/pipeline-dashboard/presentation/stores/usePipelineDashboardStore.ts';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';

export const usePipelineDashboard = () => {
  const { pipelines, loading, error, setPipelines, setLoading, setError } =
    usePipelineDashboardStore();

  const getPipelines = useDependencies().pipelineDashboard.getPipelinesUseCase;

  const fetchPipelines = useCallback(async () => {
    setLoading(true);
    setError('');

    const result = await getPipelines.execute();

    result.fold(
      data => {
        setPipelines(data.pipelines);
      },
      err => {
        err.log();
        setError(err.message);
      },
    );

    setLoading(false);
  }, [getPipelines, setPipelines, setLoading, setError]);

  return { pipelines, loading, error, fetchPipelines };
};
