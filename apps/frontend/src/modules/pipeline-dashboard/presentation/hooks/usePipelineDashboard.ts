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

    try {
      const res = await getPipelines.execute();

      if (res.ok) setPipelines(res.value.pipelines);
      else setError(res.error.message);
    } catch (error) {
      setError('Failed to fetch pipelines.' + error);
    } finally {
      setLoading(false);
    }
  }, [setLoading, setError, getPipelines, setPipelines]);

  return {
    pipelines,
    loading,
    error,
    fetchPipelines,
  };
};
