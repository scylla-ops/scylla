import { useMemo } from 'react';
import { useParams } from 'react-router-dom';
import { Trans } from '@lingui/react/macro';
import { createDefaultScript } from '@/modules/features/pipeline/presentation/utils/create-default-script.ts';
import { useCreatePipeline } from '@/modules/features/pipeline/presentation/hooks/use-create-pipeline.ts';
import { PipelineEditor } from '@/modules/features/pipeline/presentation/ui/editor/PipelineEditor.tsx';

export const PipelineCreationPage = () => {
  const { projectId } = useParams();
  const createPipeline = useCreatePipeline();

  const initialScript = useMemo(
    () => (projectId ? createDefaultScript(projectId) : undefined),
    [projectId],
  );

  if (!projectId)
    return (
      <p>
        <Trans>Select a project first</Trans>
      </p>
    );

  return (
    <div className='flex h-full flex-col gap-4'>
      <PipelineEditor
        mode='create'
        submitLabel='Create'
        projectId={projectId}
        initialScript={initialScript}
        onSubmit={({ name, steps }) => createPipeline.mutate({ name, projectId, nodes: steps })}
        isSubmitPending={createPipeline.isPending}
      />
    </div>
  );
};
