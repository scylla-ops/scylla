import { createQueries, createQuery } from '@platform/query';
import { jobsByPipelinesQueries, type JobEntity } from '@/modules/features/jobs';
import { createPagination } from '@shared/presentation/state/pagination.svelte.ts';
import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
import { ScyllaError } from '@shared/utils/scylla-result.ts';
import { pipelineMessages } from './pipeline.messages.ts';
import { pipelineQueries } from './pipeline.queries.ts';

/** The shape `jobsByPipelinesQueries().combine` reads off each fanned-out query. */
type HistoryResult = {
  data?: { pipelineId: string; jobs: JobEntity[] };
  isLoading: boolean;
  isError: boolean;
};

/**
 * The project dashboard: one page of pipelines, each with its recent run
 * history beside it.
 *
 * Two reads, and the second depends on the first — the history fan-out is one
 * query per pipeline on the page, so it cannot be declared until the page is
 * known. `jobsByPipelinesQueries` comes from `features/jobs` through its public
 * API, gate included: `ListJobsByPipeline` is enforced per project, so without
 * the grant nothing is asked and `canListJobs` is what the history cells read
 * to say "not allowed to look" rather than "failed to load".
 */
export const createPipelineDashboard = (projectId: () => string) => {
  const pagination = createPagination({ responsive: true });

  const pipelinesQuery = createQuery(() =>
    pipelineQueries.byProject(projectId(), pagination.paginationParams, {
      enabled: pagination.isPageSizeReady,
    }),
  );

  const pipelines = $derived(pipelinesQuery.data?.items);
  const pipelineIds = $derived(pipelines?.map(pipeline => pipeline.id) ?? []);

  // The clamp inside `updatePaginationInfo` is remembered state — the page the
  // reader is on can stop existing when the last row of the last page is
  // deleted — so this is synchronisation with the server's answer, not a mirror
  // of it. The React hook needed the same effect for the same reason.
  $effect(() => {
    pagination.updatePaginationInfo(pipelinesQuery.data?.pagination);
  });

  const fanOut = $derived(jobsByPipelinesQueries(pipelineIds));
  const jobResults = createQueries(() => ({ queries: fanOut.queries }));

  /**
   * Folded here rather than handed to `createQueries` as its `combine`.
   * TanStack's Svelte binding wraps a combined result in a raw ref that
   * re-exposes its top-level keys, which empties a `Map` silently — the trap
   * `grant-target-labels.svelte.ts` documents. An array of results survives it.
   */
  const jobs = $derived(fanOut.combine([...jobResults] as HistoryResult[]));

  return {
    pagination,
    get pipelines() {
      return pipelines;
    },
    get pipelineIds() {
      return pipelineIds;
    },
    get totalCount() {
      return pagination.paginationInfo?.totalCount ?? pipelineIds.length;
    },
    get isError() {
      return pipelinesQuery.isError;
    },
    get errorMessage() {
      return pipelinesQuery.error instanceof ScyllaError
        ? pipelinesQuery.error.userMessage()
        : t(pipelineMessages.loadError);
    },
    get jobsByPipelineId() {
      return jobs.jobsByPipelineId;
    },
    get isJobsLoading() {
      return jobs.isJobsLoading;
    },
    get isJobsError() {
      return jobs.isJobsError;
    },
    get canListJobs() {
      return fanOut.canListJobs;
    },
  };
};

export type PipelineDashboard = ReturnType<typeof createPipelineDashboard>;
