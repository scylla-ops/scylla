import { useMemo } from 'react';
import { useParams } from 'react-router-dom';
import { usePipeline } from '@/modules/features/pipeline/presentation/hooks/use-pipeline.ts';
import { useUpdatePipeline } from '@/modules/features/pipeline/presentation/hooks/use-update-pipeline.ts';
import { ErrorState } from '@shared/presentation/ui/feedback/ErrorState.tsx';
import { PipelineEditor } from '@/modules/features/pipeline/presentation/ui/editor/PipelineEditor.tsx';

export const PipelineUpdatePage = () => {
  const { pipelineId } = useParams();
  const { pipeline, isLoading, isError } = usePipeline(pipelineId ?? '');
  const updatePipeline = useUpdatePipeline();

  const initialScript = useMemo(
    () =>
      pipeline
        ? JSON.stringify(
            { name: pipeline.name, projectId: pipeline.projectId, nodes: pipeline.nodes },
            null,
            2,
          )
        : undefined,
    [pipeline],
  );

  if (isLoading) return <>Loading...</>;
  if (isError) return <ErrorState message='Failed to load pipeline' />;

  return (
    <div className='flex h-full flex-col gap-4 overflow-hidden'>
      <PipelineEditor
        mode='edit'
        submitLabel='Save'
        projectId={pipeline?.projectId}
        initialScript={initialScript}
        onSubmit={({ name, steps }) =>
          pipelineId && updatePipeline.mutate({ id: pipelineId, name, nodes: steps })
        }
        isSubmitPending={updatePipeline.isPending}
      />
    </div>
  );
};
