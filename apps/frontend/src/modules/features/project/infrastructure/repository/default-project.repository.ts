import type { ProjectRemoteDataSource } from '@/modules/features/project/infrastructure/repository/data-sources/project-remote.data-source.ts';
import type { ProjectRepository } from '@/modules/features/project/domain/repository/project.repository.ts';
import type { PaginationParams } from '@shared/domain/models/pagination.model.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { Project } from '@/modules/features/project/domain/models/project.model.ts';
import { GrpcProjectMapper } from '@/modules/features/project/infrastructure/repository/mappers/grpc-project.mapper.ts';

export class DefaultProjectRepository implements ProjectRepository {
  constructor(private readonly _remoteDataSource: ProjectRemoteDataSource) {}

  async getByOrganizationId(organizationId: string, pagination?: PaginationParams) {
    return (await this._remoteDataSource.getByOrganizationId(organizationId, pagination)).map(
      GrpcProjectMapper.toDomainList,
    );
  }

  async create(name: string, organizationId: string): Promise<ScyllaResult<Project>> {
    return (await this._remoteDataSource.create(name, organizationId)).map(
      GrpcProjectMapper.toDomain,
    );
  }

  async update(projectId: string, name?: string, description?: string): Promise<ScyllaResult<Project>> {
    return (await this._remoteDataSource.update(projectId, name, description)).map(
      GrpcProjectMapper.toDomain,
    );
  }

  async delete(projectId: string): Promise<ScyllaResult<void>> {
    return this._remoteDataSource.delete(projectId);
  }
}
