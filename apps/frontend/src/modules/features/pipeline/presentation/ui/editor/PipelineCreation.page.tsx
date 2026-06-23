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
      <Tabs key={'editor'} defaultValue={'blueprint'} className={'h-full flex flex-col gap-4'}>
        <div className='flex items-center justify-between gap-4'>
          <BackButton iconOnly onClick={() => goBack()} />
          <PipelineEditorHeader onSubmit={handleCreate} submitLabel='Create' isPending={createPipeline.isPending} />
        </div>
        <TabsContent value='scripting' className={'h-full'} forceMount>
          <Card className={'h-full p-0'}>
            <ReactCodeMirror
              value={script}
              onChange={value => setScript(value)}
              className='h-full'
              height='100%'
              extensions={[StreamLanguage.define(json), codeMirrorTheme]}
            />
          </Card>
        </TabsContent>
        <TabsContent value='blueprint' className='h-full'>
          <Card className='h-full p-0'>
            <PipelineBlueprint
              steps={steps}
              pipelineName={pipelineName}
              onStepsChange={handleStepsChange}
              onNameChange={handleNameChange}
            />
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
};
