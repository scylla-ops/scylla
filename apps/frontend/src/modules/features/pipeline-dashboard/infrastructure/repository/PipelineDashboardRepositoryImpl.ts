import type { PipelineDashboardRepository } from '@/modules/features/pipeline-dashboard/domain/repository/PipelineDashboardRepository.ts';
import type { ListPipelinesResponse } from '@/generated/pipeline.ts';
import type { PipelineDashboardRemoteDataSource } from '@/modules/features/pipeline-dashboard/infrastructure/repository/data-sources/PipelineDashboardRemoteDataSource.ts';
import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';

export class PipelineDashboardRepositoryImpl implements PipelineDashboardRepository {
  constructor(private readonly remoteDataSource: PipelineDashboardRemoteDataSource) {}

  public async getAll(): Promise<ScyllaResult<ListPipelinesResponse>> {
    return this.remoteDataSource.getAll();
  }

  public async deleteById(id: string): Promise<ScyllaResult<void>> {
    return this.remoteDataSource.deleteById(id);
  }
}
