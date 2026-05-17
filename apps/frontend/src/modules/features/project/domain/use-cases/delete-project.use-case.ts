import type { ProjectRepository } from '@/modules/features/project/domain/repository/project.repository.ts';

export class DeleteProjectUseCase {
  constructor(private readonly _repository: ProjectRepository) {}

  public execute = (projectId: string) => this._repository.delete(projectId);
}

