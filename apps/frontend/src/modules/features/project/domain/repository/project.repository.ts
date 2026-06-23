import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { ProjectEntity } from '@/modules/features/project/domain/entities/project.entity.ts';
import type { ProjectList } from '@/modules/features/project/domain/structs/project.struct.ts';
import type { PaginationParams } from '@shared/domain/structs/pagination.struct.ts';

export interface ProjectRepository {
  getByOrganizationId: (
    organizationId: string,
    pagination?: PaginationParams,
  ) => Promise<ScyllaResult<ProjectList>>;
  create: (
    name: string,
    organizationId: string,
    description?: string,
  ) => Promise<ScyllaResult<ProjectEntity>>;
  update: (
    projectId: string,
    name?: string,
    description?: string,
  ) => Promise<ScyllaResult<ProjectEntity>>;
  delete: (projectId: string) => Promise<ScyllaResult<void>>;
}
