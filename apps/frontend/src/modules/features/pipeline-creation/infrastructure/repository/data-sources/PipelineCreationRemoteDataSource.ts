import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { PipelineResponse } from '@/generated/pipeline.ts';

export interface PipelineCreationRemoteDataSource {
  createPipeline: (content: string) => Promise<ScyllaResult<void>>;
  getPipelineById: (id: string) => Promise<ScyllaResult<PipelineResponse>>;
}
