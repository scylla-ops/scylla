import type { PaginationParams } from '@shared/domain/structs/pagination.struct.ts';

/**
 * The project list of one organization, for one page.
 *
 * A factory rather than a literal so every reader and every invalidation agree:
 * the same organization asked for the same page is the same cache entry, whether
 * the caller is the project list, the dashboard overview or the grant-label
 * lookup. They used to spell three different keys for this one request.
 */
export const PROJECTS_QUERY_KEY = (
  organizationId: string | null,
  pagination?: PaginationParams,
) => ['projects', organizationId, pagination] as const;

/** Prefix matching every page of every organization — for broad invalidation. */
export const PROJECTS_QUERY_ROOT = ['projects'] as const;

/**
 * One request covers an organization's whole project list for the lookups that
 * need names rather than a page (dashboard overview, grant target labels).
 */
export const PROJECTS_LOOKUP_PAGE: PaginationParams = { page: 1, pageSize: 100 };
