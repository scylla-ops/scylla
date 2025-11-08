import { useCallback } from 'react';
import { PipelineStore } from '@/modules/pipeline-dashboard/presentation/stores/pipelineStore.ts';
import { useDependencies } from '@/modules/core/presentation/hooks/useDependencies';

export const usePipelineDashboard = () => {
    const deps = useDependencies();
    const store = new PipelineStore();

    const fetchPipelineStats = useCallback(async (id: string) => {
        store.setLoading(true);
        store.setError("");

        try {
            const result = await deps.pipelineDashboardUseCase.execute(id);
            if (result.ok) {
                store.setPipeline(result.value);
            } else {
                store.setError(result.error);
            }
        } catch (error) {
            store.setError('Failed to fetch pipeline stats.' + error);
        } finally {
            store.setLoading(false);
        }
    }, [deps.getMarketplaceUseCase, store]);

    return {
        pipelines: store.getPipeline,
        loading: store.isLoading,
        error: store.getError,
        fetchPipelineStats,
    };
};