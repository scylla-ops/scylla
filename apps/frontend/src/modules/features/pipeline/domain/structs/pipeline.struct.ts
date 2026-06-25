export interface PipelineIdentity {
  id: string;
  projectId: string;
  name: string;
}

export interface PipelineMetadata extends PipelineIdentity {
  nodeCount: number;
  createdAt: string;
  updatedAt: string;
}

export type Shell = 'sh' | 'bash';

/**
 * A single environment variable on a pipeline step. Its value is either an
 * inline literal or a reference (by name) to a project-scoped secret resolved at
 * dispatch — the secret value is never carried in the client.
 */
export type EnvEntry =
  | { key: string; kind: 'literal'; value: string }
  | { key: string; kind: 'secret'; secretRef: string };

interface PipelineStepBase {
  id: string;
  deps: string[];
  workingDir?: string;
  env: EnvEntry[];
}

export interface ExecPipelineStep extends PipelineStepBase {
  kind: 'exec';
  command: string;
  args: string[];
}

export interface ScriptPipelineStep extends PipelineStepBase {
  kind: 'script';
  script: string;
  shell: Shell;
}

export type PipelineStep = ExecPipelineStep | ScriptPipelineStep;
