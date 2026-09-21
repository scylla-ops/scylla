import { queryOptions } from '@platform/query';
import { getModuleDomain } from '@platform/di';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import type { PipelineMetadata } from '../domain/structs/pipeline.struct.ts';
import type { PipelineModule } from '../pipeline.module.ts';
import {
  ORGANIZATION_PIPELINES_QUERY_KEY,
  PIPELINES_LOOKUP_PAGE,
} from './hooks/pipelines.query-keys.ts';

/**
 * The reads of this module that **cross a barrel**, declared as plain data.
 *
 * `pipeline` is the last React feature and only migrates in Phase 5, but
 * `dashboard` — which reads its organization-wide list — went Svelte in Phase 4
 * and cannot call a hook. A `queryOptions` object has no framework in it, so
 * `useQuery` and `createQuery` take the same one and share one cache entry;
 * `use-organization-pipelines.ts` is now the React binding of what is declared
 * here, and disappears with the rest of the module.
 *
 * The repository is resolved per call, never at module load.
 */
const repository = () =>
  getModuleDomain<typeof PipelineModule.domain>('pipeline').pipelineRepository;

export const pipelineQueries = {
  /**
   * Every pipeline of one organization, in a single request.
   *
   * The replacement for the per-project fan-out the dashboard used to run: that
   * cost one request and one cache entry per project and — because
   * `ListPipelinesByProject` is enforced per project — one error toast for
   * every project the caller could not read. `ListOrganizationPipelines` is
   * scoped server-side, so there is nothing to gate client-side.
   */
  byOrganization: (organizationId: string | null) =>
    queryOptions<PaginatedList<PipelineMetadata>>({
      queryKey: ORGANIZATION_PIPELINES_QUERY_KEY(organizationId, PIPELINES_LOOKUP_PAGE),
      enabled: !!organizationId,
      queryFn: async () =>
        (
          await repository().getMetadataByOrganizationId(organizationId!, PIPELINES_LOOKUP_PAGE)
        ).unwrap(),
      staleTime: 30_000,
    }),
};

/**
 * The shape both consumers derive from one page of pipeline metadata.
 *
 * A pure fold rather than a second query: the window is a single page, so
 * whether it covers the whole list is something the page itself answers.
 */
export const asPipelineFeed = (data: PaginatedList<PipelineMetadata> | undefined) => {
  const pipelines = data?.items ?? [];
  const totalCount = data?.pagination.totalCount ?? 0;

  return {
    pipelines,
    totalCount,
    /** True when more pipelines exist than the single page fetched. */
    isPartialWindow: totalCount > pipelines.length,
  };
};
