import type {
  Project,
  ProjectList,
} from '@/modules/features/project/domain/models/project.model.ts';
import type { ListProjectsResponse, ProjectResponse } from '@/generated/project.ts';
import type { PaginationInfo } from '@shared/domain/models/pagination.model.ts';

export class GrpcProjectMapper {
  static toDomain(grpcProject: ProjectResponse): Project {
    return {
      id: grpcProject.projectId,
      name: grpcProject.name,
      description: grpcProject.description,
    };
  }
  static toDomainList(grpcProjects: ListProjectsResponse): ProjectList {
    return {
      projects: grpcProjects.projects.map(GrpcProjectMapper.toDomain),
      pagination: grpcProjects.pagination as PaginationInfo,
    };
  }
}
