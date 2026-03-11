import type { ListPipelinesResponse, PipelineResponse } from '@/generated/pipeline';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';

export interface PipelineDashboardStore {
  getPipelines(): Promise<ScyllaResult<ListPipelinesResponse>>;
}
