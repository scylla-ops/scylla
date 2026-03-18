import type { ListPipelinesResponse } from '@/generated/pipeline.ts';
import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';

export interface PipelineDashboardRepository {
  getAll(): Promise<ScyllaResult<ListPipelinesResponse>>;
}
