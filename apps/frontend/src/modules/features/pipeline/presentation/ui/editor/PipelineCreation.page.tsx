import ReactCodeMirror from '@uiw/react-codemirror';
import { StreamLanguage } from '@codemirror/language';
import { Tabs, TabsContent } from '@shadcn/tabs.tsx';
import { Card } from '@shadcn';
import { json } from '@codemirror/legacy-modes/mode/javascript';
import { useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { Trans } from '@lingui/react/macro';
import { PipelineEditorHeader } from '@/modules/features/pipeline/presentation/ui/editor/PipelineEditorHeader.tsx';
import { useScriptStore } from '@/modules/features/pipeline/presentation/stores/use-script.store.ts';
import { createDefaultScript } from '@/modules/features/pipeline/presentation/utils/create-default-script.ts';
import { useCreatePipeline } from '@/modules/features/pipeline/presentation/hooks/use-create-pipeline.ts';
import { codeMirrorTheme } from '@/modules/features/pipeline/presentation/utils/code-mirror-theme.ts';

export const PipelineCreationPage = () => {
  const { script, setScript } = useScriptStore(state => state);
  const { projectId } = useParams();
  const createPipeline = useCreatePipeline();

  useEffect(() => {
    if (projectId) {
      setScript(createDefaultScript(projectId));
    }
  }, [projectId, setScript]);

  if (!projectId)
    return (
      <p>
        <Trans>Select a project first</Trans>
      </p>
    );

  const handleCreate = () => {
    createPipeline.mutate(script);
  };

  return (
    <Tabs key={'scripting'} defaultValue={'scripting'} className={'h-full flex flex-col gap-4'}>
      <PipelineEditorHeader onSubmit={handleCreate} submitLabel='Create' />
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
