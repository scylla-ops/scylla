import { useEffect } from 'react';
import { PipelineChart } from "./PipelineChart"
import { StatusCard } from "./StatusCard"
import { usePipelineDashboard } from '../hooks/usePipelineDashboard';

export const DashboardPipelinePage = () => {
    const { pipelines, loading, error, fetchPipelines } = usePipelineDashboard();

    useEffect(() => {
        fetchPipelines();
    }, [fetchPipelines]);

    if (loading) {
        return <div className="flex items-center justify-center h-screen">Loading pipelines...</div>;
    }

    if (error) {
        return <div className="flex items-center justify-center h-screen text-red-500">Error: {String(error)}</div>;
    }

    return (
        <>
            <div className="flex-1 p-6 space-x-6 flex">
                {pipelines.length > 0 ? (
                    pipelines.map((pipeline) => (
                        <StatusCard key={pipeline.pipelineId} pipeline={pipeline} />
                    ))
                ) : (
                    <div className="text-gray-500">No pipelines found</div>
                )}
            </div>
            <PipelineChart />
        </>
    )
}