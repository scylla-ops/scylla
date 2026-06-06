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

interface PipelineStepBase {
  id: string;
  deps: string[];
  workingDir?: string;
  env: Record<string, string>;
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

export interface Pipeline extends PipelineIdentity {
  nodes: PipelineStep[];
}
