import { useCallback, useMemo } from 'react';
import { useScriptStore } from '@/modules/features/pipeline/presentation/stores/use-script.store.ts';
import type { PipelineStep } from '@/modules/features/pipeline/domain/models/pipeline.model.ts';

interface UsePipelineScriptParams {
  /** Project ID used as fallback when the script is unparseable */
  projectId?: string;
}

export function usePipelineScript({ projectId = '' }: UsePipelineScriptParams = {}) {
  const { script, setScript } = useScriptStore(state => state);

  const pipelineName: string = useMemo(() => {
    try {
      return JSON.parse(script).name ?? 'my-pipeline';
    } catch {
      return 'my-pipeline';
    }
  }, [script]);

  const steps: PipelineStep[] = useMemo(() => {
    try {
      const parsed = JSON.parse(script);
      return (parsed.nodes ?? []).map((n: Record<string, unknown>) => ({
        id: (n.id as string) ?? '',
        deps: (n.deps as string[]) ?? [],
        command: (n.command as string) ?? '',
        args: (n.args as string[]) ?? [],
      }));
    } catch {
      return [];
    }
  }, [script]);

  const handleStepsChange = useCallback(
    (newSteps: PipelineStep[]) => {
      const nodes = newSteps.map(s => ({
        id: s.id,
        deps: s.deps,
        command: s.command,
        args: s.args,
      }));

      try {
        const parsed = JSON.parse(script);
        parsed.nodes = nodes;
        setScript(JSON.stringify(parsed, null, 2));
      } catch {
        const obj = { name: 'my-pipeline', projectId, nodes };
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

  return {
    script,
    setScript,
    pipelineName,
    steps,
    handleStepsChange,
    handleNameChange,
  };
}

