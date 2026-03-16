import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';
import type { ListProjectsResponse, ProjectResponse } from '@/generated/project.ts';

export interface ProjectRepository {
  getAll: () => Promise<ScyllaResult<ListProjectsResponse>>;
  create: (name: string, organizationId: string) => Promise<ScyllaResult<ProjectResponse>>;
}
