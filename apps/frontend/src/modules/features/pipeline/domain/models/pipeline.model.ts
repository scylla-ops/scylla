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

export interface PipelineStep {
  id: string;
  deps: string[];
  command: string;
  args: string[];
}

export interface Pipeline extends PipelineIdentity {
  nodes: PipelineStep[];
}
