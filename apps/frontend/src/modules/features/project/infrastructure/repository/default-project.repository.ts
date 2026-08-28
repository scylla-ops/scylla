import type { ProjectRemoteDataSource } from '@/modules/features/project/infrastructure/repository/data-sources/project-remote.data-source.ts';
import type { ProjectRepository } from '@/modules/features/project/domain/repository/project.repository.ts';
import type { PaginationParams } from '@shared/domain/structs/pagination.struct.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { ProjectEntity } from '@/modules/features/project/domain/entities/project.entity.ts';
import type { ProjectMember } from '@/modules/features/project/domain/structs/project-member.struct.ts';
import { GrpcProjectMapper } from '@/modules/features/project/infrastructure/repository/mappers/grpc-project.mapper.ts';
import { GrpcProjectMemberMapper } from '@/modules/features/project/infrastructure/repository/mappers/grpc-project-member.mapper.ts';

export class DefaultProjectRepository implements ProjectRepository {
  constructor(private readonly _remoteDataSource: ProjectRemoteDataSource) {}

  async getByOrganizationId(organizationId: string, pagination?: PaginationParams) {
    return (await this._remoteDataSource.getByOrganizationId(organizationId, pagination)).map(
      GrpcProjectMapper.toDomainList,
    );
  }

  async listMembers(projectId: string): Promise<ScyllaResult<ProjectMember[]>> {
    return (await this._remoteDataSource.listMembers(projectId)).map(members =>
      members.map(GrpcProjectMemberMapper.toDomain),
    );
  }

  async create(
    name: string,
    organizationId: string,
    description?: string,
  ): Promise<ScyllaResult<ProjectEntity>> {
    return (await this._remoteDataSource.create(name, organizationId, description)).map(
      GrpcProjectMapper.toDomain,
    );
  }

  async update(
    projectId: string,
    name?: string,
    description?: string,
  ): Promise<ScyllaResult<ProjectEntity>> {
    return (await this._remoteDataSource.update(projectId, name, description)).map(
      GrpcProjectMapper.toDomain,
    );
  }

  async delete(projectId: string): Promise<ScyllaResult<void>> {
    return this._remoteDataSource.delete(projectId);
  }
}
