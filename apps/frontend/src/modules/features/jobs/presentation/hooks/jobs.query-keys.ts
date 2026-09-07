import type { PaginationParams } from '@shared/domain/structs/pagination.struct.ts';

/**
 * Jobs of one pipeline.
 *
 * Lives in `jobs` rather than next to the pipeline dashboard that also reads it:
 * it is this module's cache that gets invalidated, and having pipeline own the
 * key is what used to make the two modules mutually dependent.
 */
export const JOBS_QUERY_KEY = (pipelineId: string) => ['jobs', 'pipeline', pipelineId] as const;

/**
 * Jobs of one organization — the run feed behind the dashboard.
 *
 * Sits under the same `'jobs'` root as the per-pipeline key, so invalidating
 * `['jobs']` after a run or a deletion refreshes both scopes.
 */
export const ORGANIZATION_JOBS_QUERY_KEY = (
  organizationId: string | null,
  pagination?: PaginationParams,
) => ['jobs', 'organization', organizationId, pagination] as const;

/** Prefix matching every job listing, whatever the scope. */
export const JOBS_QUERY_ROOT = ['jobs'] as const;
