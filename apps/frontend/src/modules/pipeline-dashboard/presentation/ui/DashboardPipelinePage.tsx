import { useEffect } from 'react';
import { StatusCard } from './StatusCard';
import { usePipelineDashboard } from '../hooks/usePipelineDashboard';
import { PipelineResponse } from '@/generated/pipeline.ts';

const pipelinesTest: PipelineResponse[] = [
  {
    name: 'Pipeline 1',
    projectId: '1',
    pipelineId: '1',
    nodes: [],
    createdAt: 'test',
    updatedAt: 'test',
  },
  {
    name: 'Pipeline 2',
    projectId: '1',
    pipelineId: '1',
    nodes: [],
    createdAt: 'test',
    updatedAt: 'test',
  },
  {
    name: 'Pipeline 3',
    projectId: '1',
    pipelineId: '1',
    nodes: [],
    createdAt: 'test',
    updatedAt: 'test',
  },
  {
    name: 'Pipeline 4',
    projectId: '1',
    pipelineId: '1',
    nodes: [],
    createdAt: 'test',
    updatedAt: 'test',
  },
  {
    name: 'Pipeline 5',
    projectId: '1',
    pipelineId: '1',
    nodes: [],
    createdAt: 'test',
    updatedAt: 'test',
  },
  {
    name: 'Pipeline 6',
    projectId: '1',
    pipelineId: '1',
    nodes: [],
    createdAt: 'test',
    updatedAt: 'test',
  },
];

export const DashboardPipelinePage = () => {
  const { loading, error, fetchPipelines } = usePipelineDashboard();

  useEffect(() => {
    //fetchPipelines().finally();
  }, [fetchPipelines]);

  if (loading) {
    return <div className='flex items-center justify-center h-screen'>Loading pipelines...</div>;
  }

  if (error) {
    return (
      <div className='flex items-center justify-center h-screen text-red-500'>
        Error: {String(error)}
      </div>
    );
  }

  return (
    <>
      <div className='grid grid-cols-1 md:grid-cols-3 gap-4 w-full'>
        {pipelinesTest.length > 0 ? (
          pipelinesTest.map(pipeline => (
            <StatusCard key={pipeline.pipelineId} pipeline={pipeline} />
          ))
        ) : (
          <div className='text-gray-500'>No pipelines found</div>
        )}
      </div>
    </>
  );
};
