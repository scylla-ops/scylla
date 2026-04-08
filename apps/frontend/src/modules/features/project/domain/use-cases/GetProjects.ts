import type { ProjectRepository } from '@/modules/features/project/domain/repository/ProjectRepository.ts';
import type { PaginationParams } from '@/modules/shared/domain/types/Pagination.ts';

export class GetProjects {
  constructor(private readonly _repository: ProjectRepository) {}

  public execute = (organizationId: string, pagination?: PaginationParams) =>
    this._repository.getByOrganizationId(organizationId, pagination);
}
