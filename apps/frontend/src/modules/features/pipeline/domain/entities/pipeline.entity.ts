import type {
  PipelineIdentity,
  PipelineStep,
} from '@/modules/features/pipeline/domain/structs/pipeline.struct.ts';

export interface PipelineEntity extends PipelineIdentity {
  nodes: PipelineStep[];
}
