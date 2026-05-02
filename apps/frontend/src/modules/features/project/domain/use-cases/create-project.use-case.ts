import type { ProjectRepository } from '@/modules/features/project/domain/repository/project.repository.ts';

export class CreateProjectUseCase {
  constructor(private readonly _repository: ProjectRepository) {}

  public execute = (name: string, organizationId: string) =>
    this._repository.create(name, organizationId);
}
