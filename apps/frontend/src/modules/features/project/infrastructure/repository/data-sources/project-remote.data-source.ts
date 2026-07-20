import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  ListOrganizationProjectsResponse,
  Project,
} from '@/generated/scylla/project/v1/project.ts';
import type { PaginationParams } from '@shared/domain/structs/pagination.struct.ts';

export interface ProjectRemoteDataSource {
  getByOrganizationId: (
    organizationId: string,
    pagination?: PaginationParams,
  ) => Promise<ScyllaResult<ListOrganizationProjectsResponse>>;
  create: (
    name: string,
    organizationId: string,
    description?: string,
  ) => Promise<ScyllaResult<Project>>;
  update: (
    projectId: string,
    name?: string,
    description?: string,
  ) => Promise<ScyllaResult<Project>>;
  delete: (projectId: string) => Promise<ScyllaResult<void>>;
}
