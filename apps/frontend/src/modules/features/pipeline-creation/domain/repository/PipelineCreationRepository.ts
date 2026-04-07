import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';

export interface PipelineCreationRepository {
  createPipeline: (content: string) => Promise<ScyllaResult<void>>;
}
