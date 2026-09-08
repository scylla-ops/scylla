import type { ProjectEntity } from '@/modules/features/project/domain/entities/project.entity.ts';
import type { ProjectList } from '@/modules/features/project/domain/structs/project.struct.ts';
import type {
  ListOrganizationProjectsResponse,
  Project,
} from '@/generated/scylla/project/v1/project.ts';
import type { PaginationInfo } from '@shared/domain/structs/pagination.struct.ts';
import { idValue } from '@shared/infrastructure/grpc/wrappers.ts';

export class GrpcProjectMapper {
  static toDomain(grpcProject: Project): ProjectEntity {
    return {
      id: idValue(grpcProject.projectId),
      name: grpcProject.name,
      description: grpcProject.description,
    };
  }
  static toDomainList(grpcProjects: ListOrganizationProjectsResponse): ProjectList {
    return {
      projects: grpcProjects.projects.map(GrpcProjectMapper.toDomain),
      pagination: grpcProjects.pagination as PaginationInfo,
    };
  }
}
