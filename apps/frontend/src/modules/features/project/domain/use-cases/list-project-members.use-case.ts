import type { ProjectRepository } from '@/modules/features/project/domain/repository/project.repository.ts';
import type { ProjectMember } from '@/modules/features/project/domain/structs/project-member.struct.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

export class ListProjectMembersUseCase {
  constructor(private readonly _repository: ProjectRepository) {}

  public execute = (projectId: string): Promise<ScyllaResult<ProjectMember[]>> =>
    this._repository.listMembers(projectId);
}
