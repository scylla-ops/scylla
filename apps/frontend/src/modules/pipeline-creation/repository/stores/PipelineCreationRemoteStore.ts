import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';
import type { PipelineResponse } from '@/generated/pipeline.ts';

export interface PipelineCreationRemoteStore {
  createPipeline: (content: string) => Promise<ScyllaResult<void>>;
  getPipelineById: (id: string) => Promise<ScyllaResult<PipelineResponse>>;
}
