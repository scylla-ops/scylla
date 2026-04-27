import type { PipelineRepository } from '@/modules/features/pipeline-dashboard/domain/repository/pipeline.repository.ts';
import type { ListPipelinesResponse } from '@/generated/pipeline.ts';
import type { PipelineRemoteDataSource } from '@/modules/features/pipeline-dashboard/infrastructure/repository/data-sources/pipeline-remote.data-source.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PaginationParams } from '@/modules/shared/domain/models/pagination.model.ts';

export class DefaultPipelineRepository implements PipelineRepository {
  constructor(private readonly remoteDataSource: PipelineRemoteDataSource) {}

  public async getByProjectId(
    projectId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<ListPipelinesResponse>> {
    return this.remoteDataSource.getByProjectId(projectId, pagination);
  }

  public async deleteById(id: string): Promise<ScyllaResult<void>> {
    return this.remoteDataSource.deleteById(id);
  }

  public async run(id: string): Promise<ScyllaResult<void>> {
    return this.remoteDataSource.run(id);
  }

  public async create(content: string): Promise<ScyllaResult<void>> {
    return this.remoteDataSource.create(content);
  }
}
