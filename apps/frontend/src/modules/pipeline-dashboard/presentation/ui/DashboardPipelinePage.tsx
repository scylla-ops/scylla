import { useEffect } from 'react';
import { StatusCard } from './StatusCard';
import { usePipelineDashboard } from '../hooks/usePipelineDashboard';
import { PipelineResponse } from '@/generated/pipeline.ts';

const pipelinesTest: PipelineResponse[] = [
  {
    name: 'lala',
    projectId: '1',
    pipelineId: '1',
    nodes: [],
    createdAt: 'test',
    updatedAt: 'test',
  },
  {
    name: 'lala',
    projectId: '1',
    pipelineId: '1',
    nodes: [],
    createdAt: 'test',
    updatedAt: 'test',
  },
  {
    name: 'lala',
    projectId: '1',
    pipelineId: '1',
    nodes: [],
    createdAt: 'test',
    updatedAt: 'test',
  },
  {
    name: 'lala',
    projectId: '1',
    pipelineId: '1',
    nodes: [],
    createdAt: 'test',
    updatedAt: 'test',
  },
  {
    name: 'lala',
    projectId: '1',
    pipelineId: '1',
    nodes: [],
    createdAt: 'test',
    updatedAt: 'test',
  },
  {
    name: 'lala',
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
    fetchPipelines().finally();
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
      <div className='p-6 flex flex-row flex-wrap h-fit gap-4'>
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
