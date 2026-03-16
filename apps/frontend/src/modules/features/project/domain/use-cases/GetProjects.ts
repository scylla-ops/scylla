import type { ProjectRepository } from '@/modules/features/project/domain/repository/ProjectRepository.ts';

export class GetProjects {
  constructor(private readonly _repository: ProjectRepository) {}

  public execute = () => this._repository.getAll();
}
