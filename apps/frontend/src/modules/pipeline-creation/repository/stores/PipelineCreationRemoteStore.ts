import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';

export interface PipelineCreationRemoteStore {
  createPipeline: (content: string) => Promise<ScyllaResult<void>>;
}
