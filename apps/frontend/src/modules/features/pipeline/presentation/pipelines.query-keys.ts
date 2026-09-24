import type { PaginationParams } from '@shared/domain/structs/pagination.struct.ts';

export const PIPELINES_QUERY_KEY = (projectId: string, pagination?: PaginationParams) =>
  ['pipelines', projectId, pagination] as const;

/** Under the `'pipelines'` root: invalidating it refreshes both scopes. */
export const ORGANIZATION_PIPELINES_QUERY_KEY = (
  organizationId: string | null,
  pagination?: PaginationParams,
) => ['pipelines', 'organization', organizationId, pagination] as const;

export const PROJECT_PIPELINES_QUERY_ROOT = (projectId: string) =>
  ['pipelines', projectId] as const;

export const PIPELINES_QUERY_ROOT = ['pipelines'] as const;

export const PIPELINE_QUERY_KEY = (pipelineId: string) => ['pipeline', pipelineId] as const;

/** One request for a project's whole list, on the overview pages. */
export const PIPELINES_LOOKUP_PAGE: PaginationParams = { page: 1, pageSize: 100 };
