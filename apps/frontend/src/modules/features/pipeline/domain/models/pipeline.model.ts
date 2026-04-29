export interface PipelineMetadata {
  id: string;
  projectId: string;
  name: string;
  nodeCount: number;
  createdAt: string;
  updatedAt: string;
}

export interface PipelineStep {
  id: string;
  deps: string[];
  command: string;
  args: string[];
}

export interface Pipeline {
  info: PipelineMetadata;
  steps: PipelineStep[];
}
