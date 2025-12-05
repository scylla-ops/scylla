import type { ListPipelinesResponse, PipelineResponse } from '@/generated/pipeline';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';

export interface PipelineDashboardRepository {
  getPipelineStatsById(id: string): Promise<ScyllaResult<PipelineResponse>>;
  getPipelines(): Promise<ScyllaResult<ListPipelinesResponse>>;
}
