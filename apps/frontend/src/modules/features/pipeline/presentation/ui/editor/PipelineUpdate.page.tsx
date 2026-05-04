import ReactCodeMirror from '@uiw/react-codemirror';
import { StreamLanguage } from '@codemirror/language';
import { Tabs, TabsContent } from '@shadcn/tabs.tsx';
import { Card } from '@shadcn';
import { json } from '@codemirror/legacy-modes/mode/javascript';
import { useCallback, useEffect, useMemo } from 'react';
import { useParams } from 'react-router-dom';
import { PipelineEditorHeader } from '@/modules/features/pipeline/presentation/ui/editor/PipelineEditorHeader.tsx';
import { useScriptStore } from '@/modules/features/pipeline/presentation/stores/use-script.store.ts';
import { usePipeline } from '@/modules/features/pipeline/presentation/hooks/use-pipeline.ts';
import { useUpdatePipeline } from '@/modules/features/pipeline/presentation/hooks/use-update-pipeline.ts';
import { ErrorState } from '@shared/presentation/ui/ErrorState.tsx';
import type {
  Pipeline,
  PipelineStep,
} from '@/modules/features/pipeline/domain/models/pipeline.model.ts';
import { codeMirrorTheme } from '@/modules/features/pipeline/presentation/utils/code-mirror-theme.ts';
import { PipelineBlueprint } from '@/modules/features/pipeline/presentation/ui/editor/blueprint/PipelineBlueprint.tsx';
import { NODE_ID_FIELD_UPDATE } from '@/modules/features/pipeline/presentation/utils/node-id-field.ts';

export const PipelineUpdatePage = () => {
  const { script, setScript } = useScriptStore(state => state);
  const { pipelineId } = useParams();
  const { pipeline, isLoading, isError } = usePipeline(pipelineId ?? '');
  const updatePipeline = useUpdatePipeline();

  useEffect(() => {
    if (pipeline) {
      console.log('Pipeline:', pipeline);
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
      console.log(parsed);
      updatePipeline.mutate({ id: pipelineId, nodes: parsed.nodes, name: parsed.name });
    } catch (error) {
      console.error('Error parsing script:', error);
    }
  }, [script, pipelineId, updatePipeline]);

  const pipelineName: string = useMemo(() => {
    try {
      return JSON.parse(script).name ?? 'pipeline';
    } catch {
      return 'pipeline';
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
          [NODE_ID_FIELD_UPDATE]: s.id,
          deps: s.deps,
          command: s.command,
          args: s.args,
        }));
        setScript(JSON.stringify(parsed, null, 2));
      } catch {
        // If script is unparseable, create fresh
        const obj = {
          name: 'pipeline',
          projectId: '',
          nodes: steps.map(s => ({ [NODE_ID_FIELD_UPDATE]: s.id, deps: s.deps, command: s.command, args: s.args })),
        };
        setScript(JSON.stringify(obj, null, 2));
      }
    },
    [script, setScript],
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

  if (isLoading) return <>Loading...</>;
  if (isError) return <ErrorState message='Failed to load pipeline' />;

  return (
    <Tabs key={'scripting'} defaultValue={'scripting'} className={'h-full flex flex-col gap-4'}>
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
