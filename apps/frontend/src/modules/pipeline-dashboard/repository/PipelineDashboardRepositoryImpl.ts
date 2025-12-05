import type { PipelineDashboardRepository } from '@/modules/pipeline-dashboard/domain/repository/PipelineDashboardRepository.ts';
import type { ScyllaResult } from '@/modules/core/domain/ScyllaResult.ts';
import type { PipelineRecord } from '@/generated/pipeline';
import type { PipelineDashboardStore } from './store/PipelineDashboardStore';

export class PipelineDashboardRepositoryImpl implements PipelineDashboardRepository {
  constructor(private readonly store: PipelineDashboardStore) {}

  async getPipelineStatsById(id: string): Promise<ScyllaResult<PipelineRecord>> {
    return await this.store.getPipelineStatsById(id);
  }
}