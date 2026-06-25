import type { ProjectRepository } from '@/modules/features/project/domain/repository/project.repository.ts';
import type { PaginationParams } from '@shared/domain/structs/pagination.struct.ts';

export class GetProjectsUseCase {
  constructor(private readonly _repository: ProjectRepository) {}

  public execute = (organizationId: string, pagination?: PaginationParams) =>
    this._repository.getByOrganizationId(organizationId, pagination);
}
