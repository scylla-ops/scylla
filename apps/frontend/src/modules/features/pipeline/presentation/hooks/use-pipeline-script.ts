import { useCallback, useMemo } from 'react';
import { useScriptStore } from '@/modules/features/pipeline/presentation/stores/use-script.store.ts';
import type {
  EnvEntry,
  PipelineStep,
  Shell,
} from '@/modules/features/pipeline/domain/models/pipeline.model.ts';

function parseShell(value: unknown): Shell {
  return value === 'bash' ? 'bash' : 'sh';
}

/** Parse the JSON `env` array into typed EnvEntry[], tolerating partial input. */
function parseEnv(value: unknown): EnvEntry[] {
  if (!Array.isArray(value)) return [];
  return value.map((raw): EnvEntry => {
    const e = (raw ?? {}) as Record<string, unknown>;
    const key = typeof e.key === 'string' ? e.key : '';
    if (e.kind === 'secret' || typeof e.secretRef === 'string') {
      return { key, kind: 'secret', secretRef: typeof e.secretRef === 'string' ? e.secretRef : '' };
    }
    return { key, kind: 'literal', value: typeof e.value === 'string' ? e.value : '' };
  });
}

function serializeEnv(env: EnvEntry[]): Record<string, unknown>[] {
  return env.map(e =>
    e.kind === 'secret'
      ? { key: e.key, kind: 'secret', secretRef: e.secretRef }
      : { key: e.key, kind: 'literal', value: e.value },
  );
}

function parseNode(n: Record<string, unknown>): PipelineStep {
  const base = {
    id: (n.id as string) ?? '',
    deps: (n.deps as string[]) ?? [],
    workingDir: typeof n.workingDir === 'string' ? n.workingDir : undefined,
    env: parseEnv(n.env),
  };

  const hasCommand = typeof n.command === 'string';
  const hasScript = typeof n.script === 'string';
  const kind =
    n.kind === 'script' || n.kind === 'exec'
      ? n.kind
      : hasCommand
        ? 'exec'
        : hasScript
          ? 'script'
          : 'exec';

  if (kind === 'script') {
    return {
      ...base,
      kind: 'script',
      script: (n.script as string) ?? '',
      shell: parseShell(n.shell),
    };
  }
  return {
    ...base,
    kind: 'exec',
    command: (n.command as string) ?? '',
    args: (n.args as string[]) ?? [],
  };
}

function serializeNode(s: PipelineStep): Record<string, unknown> {
  const base = {
    id: s.id,
    deps: s.deps,
    workingDir: s.workingDir ?? '',
    env: serializeEnv(s.env),
  };
  if (s.kind === 'script') {
    return { ...base, kind: 'script', shell: s.shell, script: s.script };
  }
  return { ...base, kind: 'exec', command: s.command, args: s.args };
}

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
      return (parsed.nodes ?? []).map((n: Record<string, unknown>) => parseNode(n));
    } catch {
      return [];
    }
  }, [script]);

  const handleStepsChange = useCallback(
    (newSteps: PipelineStep[]) => {
      const nodes = newSteps.map(serializeNode);

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
