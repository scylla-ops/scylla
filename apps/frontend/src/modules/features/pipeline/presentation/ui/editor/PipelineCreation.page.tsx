import ReactCodeMirror from '@uiw/react-codemirror';
import { StreamLanguage } from '@codemirror/language';
import { Tabs, TabsContent } from '@shadcn/tabs.tsx';
import { Card } from '@shadcn';
import { json } from '@codemirror/legacy-modes/mode/javascript';
import { useCallback, useEffect, useMemo } from 'react';
import { useParams } from 'react-router-dom';
import { Trans } from '@lingui/react/macro';
import { PipelineEditorHeader } from '@/modules/features/pipeline/presentation/ui/editor/PipelineEditorHeader.tsx';
import { useScriptStore } from '@/modules/features/pipeline/presentation/stores/use-script.store.ts';
import { createDefaultScript } from '@/modules/features/pipeline/presentation/utils/create-default-script.ts';
import { useCreatePipeline } from '@/modules/features/pipeline/presentation/hooks/use-create-pipeline.ts';
import { codeMirrorTheme } from '@/modules/features/pipeline/presentation/utils/code-mirror-theme.ts';
import { PipelineBlueprint } from '@/modules/features/pipeline/presentation/ui/editor/blueprint/PipelineBlueprint.tsx';
import type { PipelineStep } from '@/modules/features/pipeline/domain/models/pipeline.model.ts';
import { NODE_ID_FIELD_CREATE } from '@/modules/features/pipeline/presentation/utils/node-id-field.ts';

export const PipelineCreationPage = () => {
  const { script, setScript } = useScriptStore(state => state);
  const { projectId } = useParams();
  const createPipeline = useCreatePipeline();

  useEffect(() => {
    if (projectId) {
      setScript(createDefaultScript(projectId));
    }
  }, [projectId, setScript]);

  const handleCreate = () => {
    createPipeline.mutate(script);
  };

  const pipelineName: string = useMemo(() => {
    try {
      return JSON.parse(script).name ?? 'my-pipeline';
    } catch {
      return 'my-pipeline';
    }
  }, [script]);

  const currentSteps: PipelineStep[] = useMemo(() => {
    try {
      const parsed = JSON.parse(script);
      return (parsed.nodes ?? []).map((n: Record<string, unknown>) => ({
        id: (n.nodeId as string) ?? (n.id as string) ?? '',
        deps: (n.deps as string[]) ?? [],
        command: (n.command as string) ?? '',
        args: (n.args as string[]) ?? [],
      }));
    } catch {
      return [];
    }
  }, [script]);

  const handleBlueprintChange = useCallback(
    (steps: PipelineStep[]) => {
      try {
        const parsed = JSON.parse(script);
        parsed.nodes = steps.map(s => ({
          [NODE_ID_FIELD_CREATE]: s.id,
          deps: s.deps,
          command: s.command,
          args: s.args,
        }));
        setScript(JSON.stringify(parsed, null, 2));
      } catch {
        const obj = {
          name: 'my-pipeline',
          projectId,
          nodes: steps.map(s => ({
            [NODE_ID_FIELD_CREATE]: s.id,
            deps: s.deps,
            command: s.command,
            args: s.args,
          })),
        };
        setScript(JSON.stringify(obj, null, 2));
      }
    },
    [script, setScript, projectId],
  );

  const handleNameChange = useCallback(
    (name: string) => {
      try {
        const parsed = JSON.parse(script);
        parsed.name = name;
        setScript(JSON.stringify(parsed, null, 2));
      } catch {
        /* ignore */
      }
    },
    [script, setScript],
  );

  if (!projectId)
    return (
      <p>
        <Trans>Select a project first</Trans>
      </p>
    );

  return (
    <Tabs key={'scripting'} defaultValue={'scripting'} className={'h-full flex flex-col gap-4'}>
      <PipelineEditorHeader onSubmit={handleCreate} submitLabel='Create' />
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
            steps={currentSteps}
            pipelineName={pipelineName}
            onStepsChange={handleBlueprintChange}
            onNameChange={handleNameChange}
          />
        </Card>
      </TabsContent>
    </Tabs>
  );
};
