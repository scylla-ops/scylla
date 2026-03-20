import type { ProjectRepository } from '@/modules/features/project/domain/repository/ProjectRepository.ts';

export class CreateProject {
  constructor(private readonly _repository: ProjectRepository) {}

  public execute = (name: string, organizationId: string) =>
    this._repository.create(name, organizationId);
}
