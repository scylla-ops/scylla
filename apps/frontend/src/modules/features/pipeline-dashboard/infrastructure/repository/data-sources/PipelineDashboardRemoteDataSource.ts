import type { ListPipelinesResponse } from '@/generated/pipeline.ts';
import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';

export interface PipelineDashboardRemoteDataSource {
  getAll(): Promise<ScyllaResult<ListPipelinesResponse>>;
}
