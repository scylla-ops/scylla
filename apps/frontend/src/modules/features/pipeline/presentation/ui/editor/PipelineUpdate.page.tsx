import ReactCodeMirror from '@uiw/react-codemirror';
import { StreamLanguage } from '@codemirror/language';
import { Tabs, TabsContent } from '@shadcn/tabs.tsx';
import { Card } from '@shadcn';
import { json } from '@codemirror/legacy-modes/mode/javascript';
import { useCallback, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { Trans } from '@lingui/react/macro';
import { PipelineEditorHeader } from '@/modules/features/pipeline/presentation/ui/editor/PipelineEditorHeader.tsx';
import { useScriptStore } from '@/modules/features/pipeline/presentation/stores/use-script.store.ts';
import { usePipeline } from '@/modules/features/pipeline/presentation/hooks/use-pipeline.ts';
import { useUpdatePipeline } from '@/modules/features/pipeline/presentation/hooks/use-update-pipeline.ts';
import { codeMirrorTheme } from '@/modules/features/pipeline/presentation/ui/editor/PipelineCreation.page.tsx';
import { ErrorState } from '@shared/presentation/ui/ErrorState.tsx';
import type { Pipeline } from '@/modules/features/pipeline/domain/models/pipeline.model.ts';

export const PipelineUpdatePage = () => {
  const { script, setScript } = useScriptStore(state => state);
  const { pipelineId } = useParams();
  const { pipeline, isLoading, isError } = usePipeline(pipelineId ?? '');
  const updatePipeline = useUpdatePipeline();

  useEffect(() => {
    if (pipeline) {
      try {
        const scriptObj = {
          name: pipeline.name,
          projectId: pipeline.projectId,
          nodes: pipeline.nodes,
        };
        setScript(JSON.stringify(scriptObj, null, 2));
      } catch (e) {
        console.error('Error stringify pipeline:', e);
      }
    }
  }, [pipeline, setScript]);

  const handleUpdate = useCallback(() => {
    if (!script || !pipelineId) return;
    try {
      const parsed: Pipeline = JSON.parse(script);
      updatePipeline.mutate({ id: pipelineId, nodes: parsed.nodes, name: parsed.name });
    } catch (error) {
      console.error('Error parsing script:', error);
    }
  }, [script, pipelineId, updatePipeline]);

  if (isLoading) return <>Loading...</>;
  if (isError) return <ErrorState message='Failed to load pipeline' />;

  return (
    <Tabs key={'scripting'} defaultValue={'scripting'} className={'h-full flex flex-col gap-4'}>
      <PipelineEditorHeader onSubmit={handleUpdate} submitLabel='Edit' />
      <TabsContent value='scripting' className={'h-full'}>
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
      <TabsContent value='blueprint' className='h-full flex items-center justify-center'>
        <Card className='p-8 text-center'>
          <p className='text-lg text-muted-foreground'>
            <Trans>This feature will be available soon</Trans>
          </p>
        </Card>
      </TabsContent>
    </Tabs>
  );
};
