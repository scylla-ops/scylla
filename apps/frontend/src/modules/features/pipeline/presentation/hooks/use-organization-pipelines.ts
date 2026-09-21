import { useQuery } from '@tanstack/react-query';
import { asPipelineFeed, pipelineQueries } from '../pipeline.queries.ts';

/**
 * Every pipeline of the current organization, in a single request.
 *
 * The React binding of `pipelineQueries.byOrganization`; the declaration lives
 * there so `dashboard`, which is Svelte as of Phase 4, runs the very same query
 * — same key, same cache entry. This file goes with the rest of the module in
 * Phase 5.
 */
export const useOrganizationPipelines = (organizationId: string | null) => {
  const { data, isLoading, isError } = useQuery(pipelineQueries.byOrganization(organizationId));

  return { ...asPipelineFeed(data), isLoading, isError };
};
