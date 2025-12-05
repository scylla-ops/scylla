import { useCallback } from 'react';
import { usePipelineDashboardStore } from '@/modules/pipeline-dashboard/presentation/stores/usePipelineDashboardStore.ts';
import { PipelineClient } from '@/generated/pipeline.client.ts';
import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

export const usePipelineDashboard = () => {
    const { pipelines, loading, error, setPipelines, setLoading, setError } = usePipelineDashboardStore();

    const fetchPipelines = useCallback(async () => {
        setLoading(true);
        setError("");

        try {
            const apiUrl = import.meta.env.VITE_API_URL ?? '';
            const transport = new GrpcWebFetchTransport({
                baseUrl: apiUrl,
                format: 'binary',
            });
            const client = new PipelineClient(transport);
            
            const { response } = await client.listPipelines({
                pagination: {
                    page: 1,
                    pageSize: 10,
                },
            });
            
            if (response.pipelines) {
                setPipelines(response.pipelines);
            }
        } catch (error) {
            setError('Failed to fetch pipelines.' + error);
        } finally {
            setLoading(false);
        }
    }, [setLoading, setError, setPipelines]);

    return {
        pipelines,
        loading,
        error,
        fetchPipelines,
    };
};