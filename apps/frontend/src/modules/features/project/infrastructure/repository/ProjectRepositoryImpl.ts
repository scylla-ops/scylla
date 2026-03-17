import type { ProjectRemoteDataSource } from '@/modules/features/project/infrastructure/repository/data-sources/ProjectRemoteDataSource.ts';
import type { ProjectRepository } from '@/modules/features/project/domain/repository/ProjectRepository.ts';

export class ProjectRepositoryImpl implements ProjectRepository {
  constructor(private readonly _remoteDataSource: ProjectRemoteDataSource) {}

  getAll() {
    return this._remoteDataSource.getAll();
  }

  create(name: string, organizationId: string) {
    return this._remoteDataSource.create(name, organizationId);
  }
}
