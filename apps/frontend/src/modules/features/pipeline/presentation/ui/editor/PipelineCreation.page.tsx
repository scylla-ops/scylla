import ReactCodeMirror from '@uiw/react-codemirror';
import { StreamLanguage } from '@codemirror/language';
import { Tabs, TabsContent } from '@shadcn/tabs.tsx';
import { Card } from '@shadcn';
import { json } from '@codemirror/legacy-modes/mode/javascript';
import { useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { Trans } from '@lingui/react/macro';
import { PipelineEditorHeader } from '@/modules/features/pipeline/presentation/ui/editor/PipelineEditorHeader.tsx';
import { createDefaultScript } from '@/modules/features/pipeline/presentation/utils/create-default-script.ts';
import { useCreatePipeline } from '@/modules/features/pipeline/presentation/hooks/use-create-pipeline.ts';
import { codeMirrorTheme } from '@/modules/features/pipeline/presentation/utils/code-mirror-theme.ts';
import { PipelineBlueprint } from '@/modules/features/pipeline/presentation/ui/editor/blueprint/PipelineBlueprint.tsx';
import { usePipelineScript } from '@/modules/features/pipeline/presentation/hooks/use-pipeline-script.ts';
import { BackButton } from '@shared/presentation/ui/BackButton.tsx';
import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';

export const PipelineCreationPage = () => {
  const { projectId } = useParams();
  const createPipeline = useCreatePipeline();
  const { goBack } = useScyllaNavigate();

  const { script, setScript, pipelineName, steps, handleStepsChange, handleNameChange } =
    usePipelineScript({ projectId });

  useEffect(() => {
    if (projectId) {
      setScript(createDefaultScript(projectId));
    }
  }, [projectId, setScript]);

  const handleCreate = () => {
    if (!projectId) return;
    createPipeline.mutate({ name: pipelineName, projectId, nodes: steps });
  };

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
