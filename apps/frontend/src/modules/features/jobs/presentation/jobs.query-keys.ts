import type { PaginationParams } from '@shared/domain/structs/pagination.struct.ts';

export const JOBS_QUERY_KEY = (pipelineId: string) => ['jobs', 'pipeline', pipelineId] as const;

/** Under the `'jobs'` root: invalidating it refreshes every scope. */
export const ORGANIZATION_JOBS_QUERY_KEY = (
  organizationId: string | null,
  pagination?: PaginationParams,
) => ['jobs', 'organization', organizationId, pagination] as const;

/** Under the `'jobs'` root too. */
export const JOB_QUERY_KEY = (jobId: string) => ['jobs', 'detail', jobId] as const;

export const JOBS_QUERY_ROOT = ['jobs'] as const;
