import ReactCodeMirror from '@uiw/react-codemirror';
import { StreamLanguage } from '@codemirror/language';
import { Tabs, TabsContent } from '@shadcn/tabs.tsx';
import { Card } from '@shadcn';
import { json } from '@codemirror/legacy-modes/mode/javascript';
import { useCallback, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { PipelineEditorHeader } from '@/modules/features/pipeline/presentation/ui/editor/PipelineEditorHeader.tsx';
import { usePipeline } from '@/modules/features/pipeline/presentation/hooks/use-pipeline.ts';
import { useUpdatePipeline } from '@/modules/features/pipeline/presentation/hooks/use-update-pipeline.ts';
import { ErrorState } from '@shared/presentation/ui/ErrorState.tsx';
import { codeMirrorTheme } from '@/modules/features/pipeline/presentation/utils/code-mirror-theme.ts';
import { PipelineBlueprint } from '@/modules/features/pipeline/presentation/ui/editor/blueprint/PipelineBlueprint.tsx';
import { usePipelineScript } from '@/modules/features/pipeline/presentation/hooks/use-pipeline-script.ts';

export const PipelineUpdatePage = () => {
  const { pipelineId } = useParams();
  const { pipeline, isLoading, isError } = usePipeline(pipelineId ?? '');
  const updatePipeline = useUpdatePipeline();

  const { script, setScript, pipelineName, steps, handleStepsChange, handleNameChange } =
    usePipelineScript();

  useEffect(() => {
    if (pipeline) {
      const scriptObj = {
        name: pipeline.name,
        projectId: pipeline.projectId,
        nodes: pipeline.nodes,
      };
      setScript(JSON.stringify(scriptObj, null, 2));
    }
  }, [pipeline, setScript]);

  const handleUpdate = useCallback(() => {
    if (!pipelineId) return;
    updatePipeline.mutate({ id: pipelineId, nodes: steps, name: pipelineName });
  }, [pipelineId, steps, pipelineName, updatePipeline]);

  if (isLoading) return <>Loading...</>;
  if (isError) return <ErrorState message='Failed to load pipeline' />;

  return (
    <Tabs key={'editor'} defaultValue={'blueprint'} className={'h-full flex flex-col gap-4'}>
      <PipelineEditorHeader onSubmit={handleUpdate} submitLabel='Edit' />
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
  );
};
