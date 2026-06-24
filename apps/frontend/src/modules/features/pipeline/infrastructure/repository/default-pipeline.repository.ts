import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PaginationParams } from '@/modules/shared/domain/models/pagination.model.ts';
import type { PipelineRemoteDataSource } from '@/modules/features/pipeline/infrastructure/repository/data-sources/pipeline-remote.data-source.ts';
import type { PipelineRepository } from '@/modules/features/pipeline/domain/repository/pipeline.repository.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import type {
  Pipeline,
  PipelineMetadata,
  PipelineStep,
} from '@/modules/features/pipeline/domain/models/pipeline.model.ts';
import { GrpcPipelineMapper } from '@/modules/features/pipeline/infrastructure/repository/mappers/grpc-pipeline.mapper.ts';
import { wrapId } from '@shared/infrastructure/grpc/wrappers.ts';

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

  public async create(pipeline: Omit<Pipeline, 'id'>): Promise<ScyllaResult<void>> {
    return this.remoteDataSource.create({
      name: pipeline.name,
      projectId: wrapId(pipeline.projectId),
      nodes: pipeline.nodes.map(GrpcPipelineMapper.nodeFromDomain),
    });
  }

  public async getById(id: string): Promise<ScyllaResult<Pipeline>> {
    return (await this.remoteDataSource.getById(id)).map(GrpcPipelineMapper.toDomain);
  }

  public async edit(id: string, nodes: PipelineStep[], name?: string) {
    return (
      await this.remoteDataSource.update(id, nodes.map(GrpcPipelineMapper.nodeFromDomain), name)
    ).map(GrpcPipelineMapper.toDomain);
  }
}
