import type { ListPipelinesResponse } from '@/generated/pipeline.ts';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';

export interface PipelineDashboardRemoteDataSource {
  getAll(): Promise<ScyllaResult<ListPipelinesResponse>>;
}
