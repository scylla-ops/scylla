import type { ListPipelinesResponse } from '@/generated/pipeline';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';

export interface PipelineDashboardRemoteDataSource {
  getPipelines(): Promise<ScyllaResult<ListPipelinesResponse>>;
}
