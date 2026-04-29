import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PaginationParams } from '@/modules/shared/domain/models/pagination.model.ts';
import type { PipelineRemoteDataSource } from '@/modules/features/pipeline/infrastructure/repository/data-sources/pipeline-remote.data-source.ts';
import type { PipelineRepository } from '@/modules/features/pipeline/domain/repository/pipeline.repository.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import type {
  Pipeline,
  PipelineMetadata,
} from '@/modules/features/pipeline/domain/models/pipeline.model.ts';
import { GrpcPipelineMapper } from '@/modules/features/pipeline/infrastructure/repository/mappers/grpc-pipeline.mapper.ts';

export class DefaultPipelineRepository implements PipelineRepository {
  constructor(private readonly remoteDataSource: PipelineRemoteDataSource) {}

  public async getMetadataByProjectId(
    projectId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<PaginatedList<PipelineMetadata>>> {
    return (await this.remoteDataSource.getByProjectId(projectId, pagination)).map(
      GrpcPipelineMapper.toDomainInfoList,
    );
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

  public async getById(id: string): Promise<ScyllaResult<Pipeline>> {
    return (await this.remoteDataSource.getById(id)).map(GrpcPipelineMapper.toDomain);
  }
}
