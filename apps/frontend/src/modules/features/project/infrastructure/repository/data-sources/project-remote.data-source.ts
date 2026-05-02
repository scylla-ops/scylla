import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { ListProjectsResponse, ProjectResponse } from '@/generated/project.ts';
import type { PaginationParams } from '@/modules/shared/domain/types/Pagination.ts';

export interface ProjectRemoteDataSource {
  getByOrganizationId: (
    organizationId: string,
    pagination?: PaginationParams,
  ) => Promise<ScyllaResult<ListProjectsResponse>>;
  create: (name: string, organizationId: string) => Promise<ScyllaResult<ProjectResponse>>;
  update: (projectId: string, name?: string, description?: string) => Promise<ScyllaResult<ProjectResponse>>;
  delete: (projectId: string) => Promise<ScyllaResult<void>>;
}
