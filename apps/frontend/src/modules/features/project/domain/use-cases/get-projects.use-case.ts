import type { ProjectRepository } from '@/modules/features/project/domain/repository/project.repository.ts';
import type { PaginationParams } from '@shared/domain/models/pagination.model.ts';

export class GetProjectsUseCase {
  constructor(private readonly _repository: ProjectRepository) {}

  public execute = (organizationId: string, pagination?: PaginationParams) =>
    this._repository.getByOrganizationId(organizationId, pagination);
}
