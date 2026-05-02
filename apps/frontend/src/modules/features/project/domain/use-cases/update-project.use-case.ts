import type { ProjectRepository } from '@/modules/features/project/domain/repository/project.repository.ts';

export class UpdateProjectUseCase {
  constructor(private readonly _repository: ProjectRepository) {}

  public execute = (projectId: string, name?: string, description?: string) =>
    this._repository.update(projectId, name, description);
}

