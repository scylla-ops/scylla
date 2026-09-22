import type {
  EnvEntry,
  PipelineStep,
  Shell,
} from '@/modules/features/pipeline/domain/structs/pipeline.struct.ts';

/**
 * The pipeline editor's document format: JSON in, `PipelineStep[]` out, and
 * back.
 *
 * Pure, and tested without a DOM. `use-pipeline-script.ts` held this alongside
 * the React state that drove it; the two are separable, and separating them is
 * what lets the parsing rules — which are the fiddly part, since a script is
 * hand-written and may be half-finished — be pinned without a component.
 */

/** A parsed script document, or the reason it could not be parsed. */
export interface ParsedScript {
  /** `null` for an empty editor as well as for malformed JSON. */
  document: Record<string, unknown> | null;
  /** `null` when there is nothing wrong — including for an empty editor. */
  error: string | null;
}

export const DEFAULT_PIPELINE_NAME = 'my-pipeline';

export const parseScript = (script: string): ParsedScript => {
  // An empty editor isn't an error to report — it just isn't saveable yet.
  if (!script.trim()) return { document: null, error: null };

  try {
    return { document: JSON.parse(script) as Record<string, unknown>, error: null };
  } catch (error) {
    return { document: null, error: error instanceof Error ? error.message : 'Invalid JSON' };
  }
};

const parseShell = (value: unknown): Shell => (value === 'bash' ? 'bash' : 'sh');

/** Parse the JSON `env` array into typed `EnvEntry[]`, tolerating partial input. */
const parseEnv = (value: unknown): EnvEntry[] => {
  if (!Array.isArray(value)) return [];

  return value.map((raw): EnvEntry => {
    const entry = (raw ?? {}) as Record<string, unknown>;
    const key = typeof entry.key === 'string' ? entry.key : '';

    if (entry.kind === 'secret' || typeof entry.secretRef === 'string') {
      return {
        key,
        kind: 'secret',
        secretRef: typeof entry.secretRef === 'string' ? entry.secretRef : '',
      };
    }
    return { key, kind: 'literal', value: typeof entry.value === 'string' ? entry.value : '' };
  });
};

const serializeEnv = (env: EnvEntry[]): Record<string, unknown>[] =>
  env.map(entry =>
    entry.kind === 'secret'
      ? { key: entry.key, kind: 'secret', secretRef: entry.secretRef }
      : { key: entry.key, kind: 'literal', value: entry.value },
  );

const parseNode = (node: Record<string, unknown>): PipelineStep => {
  const base = {
    id: (node.id as string) ?? '',
    deps: (node.deps as string[]) ?? [],
    workingDir: typeof node.workingDir === 'string' ? node.workingDir : undefined,
    env: parseEnv(node.env),
  };

  // An explicit `kind` wins; otherwise the field that is present says which it
  // is, and a node carrying neither is an empty command waiting to be filled in.
  const hasCommand = typeof node.command === 'string';
  const hasScript = typeof node.script === 'string';
  const kind =
    node.kind === 'script' || node.kind === 'exec'
      ? node.kind
      : hasCommand
        ? 'exec'
        : hasScript
          ? 'script'
          : 'exec';

  if (kind === 'script') {
    return {
      ...base,
      kind: 'script',
      script: (node.script as string) ?? '',
      shell: parseShell(node.shell),
    };
  }
  return {
    ...base,
    kind: 'exec',
    command: (node.command as string) ?? '',
    args: (node.args as string[]) ?? [],
  };
};

const serializeNode = (step: PipelineStep): Record<string, unknown> => {
  const base = {
    id: step.id,
    deps: step.deps,
    workingDir: step.workingDir ?? '',
    env: serializeEnv(step.env),
  };

  if (step.kind === 'script') {
    return { ...base, kind: 'script', shell: step.shell, script: step.script };
  }
  return { ...base, kind: 'exec', command: step.command, args: step.args };
};

export const stepsOf = (document: Record<string, unknown> | null): PipelineStep[] =>
  ((document?.nodes ?? []) as Record<string, unknown>[]).map(parseNode);

export const nameOf = (document: Record<string, unknown> | null): string =>
  (document?.name as string) ?? DEFAULT_PIPELINE_NAME;

/**
 * The document with its steps replaced, pretty-printed.
 *
 * Everything else the document holds survives — its name above all, which the
 * blueprint does not know about and would otherwise drop on every edit. With
 * nothing parsed yet, a minimal document is seeded instead.
 */
export const withSteps = (
  document: Record<string, unknown> | null,
  steps: PipelineStep[],
  projectId: string,
): string => {
  const base = document ?? { name: DEFAULT_PIPELINE_NAME, projectId };
  return JSON.stringify({ ...base, nodes: steps.map(serializeNode) }, null, 2);
};

/** The document renamed, pretty-printed — `null` when there is nothing to rename. */
export const withName = (
  document: Record<string, unknown> | null,
  name: string,
): string | null => (document ? JSON.stringify({ ...document, name }, null, 2) : null);
