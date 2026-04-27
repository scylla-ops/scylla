import type { ProjectRemoteDataSource } from '@/modules/features/project/infrastructure/repository/data-sources/project-remote.data-source.ts';
import type { ProjectRepository } from '@/modules/features/project/domain/repository/project.repository.ts';
import type { PaginationParams } from '@/modules/shared/domain/types/Pagination.ts';

export class DefaultProjectRepository implements ProjectRepository {
  constructor(private readonly _remoteDataSource: ProjectRemoteDataSource) {}

  getByOrganizationId(organizationId: string, pagination?: PaginationParams) {
    return this._remoteDataSource.getByOrganizationId(organizationId, pagination);
  }

  create(name: string, organizationId: string) {
    return this._remoteDataSource.create(name, organizationId);
  }
}
