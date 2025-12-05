import type { PipelineDashboardRepository } from '@/modules/pipeline-dashboard/domain/repository/PipelineDashboardRepository.ts';
import type { ListPipelinesResponse, PipelineResponse } from '@/generated/pipeline';
import type { PipelineDashboardStore } from './store/PipelineDashboardStore';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';

export class PipelineDashboardRepositoryImpl implements PipelineDashboardRepository {
  constructor(private readonly store: PipelineDashboardStore) {}

  public async getPipelineStatsById(id: string): Promise<ScyllaResult<PipelineResponse>> {
    return await this.store.getPipelineStatsById(id);
  }

  public async getPipelines(): Promise<ScyllaResult<ListPipelinesResponse>> {
    return this.store.getPipelines();
  }
}
