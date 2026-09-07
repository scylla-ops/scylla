import { useQuery } from '@tanstack/react-query';
import { usePipelineDomain } from '@/modules/features/pipeline/presentation/hooks/use-pipeline-domain.ts';
import type { PipelineMetadata } from '@/modules/features/pipeline/domain/structs/pipeline.struct.ts';
import {
  PIPELINES_LOOKUP_PAGE,
  ORGANIZATION_PIPELINES_QUERY_KEY,
} from '@/modules/features/pipeline/presentation/hooks/pipelines.query-keys.ts';

/**
 * Every pipeline of the current organization, in a single request.
 *
 * Part of the module's public API, and the replacement for the per-project
 * fan-out the dashboard used to run: that fan-out cost one request and one
 * cache entry per project, and — because `ListPipelinesByProject` is enforced
 * per project — one error toast for every project the caller could not read.
 * `ListOrganizationPipelines` is scoped server-side, so there is nothing to
 * gate client-side and nothing to deny.
 */
export const useOrganizationPipelines = (organizationId: string | null) => {
  const { pipelineRepository } = usePipelineDomain();

  const { data, isLoading, isError } = useQuery({
    queryKey: ORGANIZATION_PIPELINES_QUERY_KEY(organizationId, PIPELINES_LOOKUP_PAGE),
    queryFn: async () =>
      (
        await pipelineRepository.getMetadataByOrganizationId(
          organizationId!,
          PIPELINES_LOOKUP_PAGE,
        )
      ).unwrap(),
    enabled: !!organizationId,
    staleTime: 30_000,
  });

  const pipelines: PipelineMetadata[] = data?.items ?? [];

  return {
    pipelines,
    totalCount: data?.pagination.totalCount ?? 0,
    /** True when more pipelines exist than the single page fetched. */
    isPartialWindow: (data?.pagination.totalCount ?? 0) > pipelines.length,
    isLoading,
    isError,
  };
};
