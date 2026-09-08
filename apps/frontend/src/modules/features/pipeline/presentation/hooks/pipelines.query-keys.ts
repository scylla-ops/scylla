import type { PaginationParams } from '@shared/domain/structs/pagination.struct.ts';

/** Pipeline metadata for one project, for one page. */
export const PIPELINES_QUERY_KEY = (projectId: string, pagination?: PaginationParams) =>
  ['pipelines', projectId, pagination] as const;

/**
 * Every pipeline of one organization, in one call.
 *
 * Under the same `'pipelines'` root as the per-project key, so invalidating
 * `['pipelines']` after a create/delete refreshes both scopes.
 */
export const ORGANIZATION_PIPELINES_QUERY_KEY = (
  organizationId: string | null,
  pagination?: PaginationParams,
) => ['pipelines', 'organization', organizationId, pagination] as const;

/** Prefix matching every page of every scope — for broad invalidation. */
export const PIPELINES_QUERY_ROOT = ['pipelines'] as const;

/** One request covers a project's whole pipeline list for the overview pages. */
export const PIPELINES_LOOKUP_PAGE: PaginationParams = { page: 1, pageSize: 100 };
