import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';

export interface PipelineCreationRepository {
  createPipeline: (content: string) => Promise<ScyllaResult<void>>;
}
