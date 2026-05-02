import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  Project,
  ProjectList,
} from '@/modules/features/project/domain/models/project.model.ts';
import type { PaginationParams } from '@shared/domain/models/pagination.model.ts';

export interface ProjectRepository {
  getByOrganizationId: (
    organizationId: string,
    pagination?: PaginationParams,
  ) => Promise<ScyllaResult<ProjectList>>;
  create: (name: string, organizationId: string) => Promise<ScyllaResult<Project>>;
  update: (projectId: string, name?: string, description?: string) => Promise<ScyllaResult<Project>>;
  delete: (projectId: string) => Promise<ScyllaResult<void>>;
}
