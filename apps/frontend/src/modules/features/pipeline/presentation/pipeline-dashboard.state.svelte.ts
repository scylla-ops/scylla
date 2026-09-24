import { createQueries, createQuery } from '@platform/query';
import { jobsByPipelinesQueries, type JobEntity } from '@/modules/features/jobs';
import { createPagination } from '@shared/presentation/state/pagination.svelte.ts';
import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
import { ScyllaError } from '@shared/utils/scylla-result.ts';
import { pipelineMessages } from './pipeline.messages.ts';
import { pipelineQueries } from './pipeline.queries.ts';

type HistoryResult = {
  data?: { pipelineId: string; jobs: JobEntity[] };
  isLoading: boolean;
  isError: boolean;
};

/** One page of pipelines, each with its recent runs. The history needs `LIST_JOBS_BY_PIPELINE`: read `canListJobs`. */
export const createPipelineDashboard = (projectId: () => string) => {
  const pagination = createPagination({ responsive: true });

  const pipelinesQuery = createQuery(() =>
    pipelineQueries.byProject(projectId(), pagination.paginationParams, {
      enabled: pagination.isPageSizeReady,
    }),
  );

  const pipelines = $derived(pipelinesQuery.data?.items);
  const pipelineIds = $derived(pipelines?.map(pipeline => pipeline.id) ?? []);

  // Clamps the page when the last row of the last page is deleted.
  $effect(() => {
    pagination.updatePaginationInfo(pipelinesQuery.data?.pagination);
  });

  const fanOut = $derived(jobsByPipelinesQueries(pipelineIds));
  const jobResults = createQueries(() => ({ queries: fanOut.queries }));

  /** Folded here, not in the `combine`: the Svelte binding empties a combined `Map`. */
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
