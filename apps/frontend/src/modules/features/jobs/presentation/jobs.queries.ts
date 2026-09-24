import { getQueryClient, mutationOptions, queryOptions } from '@platform/query';
import { getModuleDomain } from '@platform/di';
import type { PaginationParams } from '@shared/domain/structs/pagination.struct.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import type { JobEntity } from '../domain/entities/job.entity.ts';
import { isActiveStatus, summarizeJobs } from '../domain/structs/jobs-summary.struct.ts';
import type { JobsModule } from '../jobs.module.ts';
import {
  JOBS_QUERY_KEY,
  JOBS_QUERY_ROOT,
  JOB_QUERY_KEY,
  ORGANIZATION_JOBS_QUERY_KEY,
} from './jobs.query-keys.ts';

// Resolved per call: tests swap the registry.
const repository = () => getModuleDomain<typeof JobsModule.domain>('jobs').jobsRepository;

const MAX_JOBS_PER_PIPELINE = 10;

/** The dashboard's figures are computed over this window: the backend does not aggregate. */
export const ORGANIZATION_JOBS_WINDOW: PaginationParams = { page: 1, pageSize: 100 };

const hasActive = (jobs: readonly JobEntity[]) => jobs.some(job => isActiveStatus(job.status));

export const jobQueries = {
  /** Polled while it runs; the polling stops at a finished status. */
  byId: (jobId: string) =>
    queryOptions<JobEntity>({
      queryKey: JOB_QUERY_KEY(jobId),
      enabled: !!jobId,
      queryFn: async () => (await repository().getById(jobId)).unwrap(),
      refetchInterval: query => {
        const status = query.state.data?.status;
        if (!status || status === 'completed' || status === 'failed') return false;
        return 3000;
      },
    }),

  /** `enabled`: wait until the page size is measured, or the first page is fetched twice. */
  byPipeline: (pipelineId: string, pagination: PaginationParams, options: { enabled?: boolean } = {}) =>
    queryOptions<PaginatedList<JobEntity>>({
      queryKey: [...JOBS_QUERY_KEY(pipelineId), pagination],
      enabled: (options.enabled ?? true) && !!pipelineId,
      queryFn: async () => (await repository().getByPipelineId(pipelineId, pagination)).unwrap(),
      staleTime: 0,
      refetchInterval: query => (hasActive(query.state.data?.items ?? []) ? 5000 : false),
    }),

  /** Polls while a job of the window is running. */
  byOrganization: (organizationId: string | null, enabled = true) =>
    queryOptions({
      queryKey: ORGANIZATION_JOBS_QUERY_KEY(organizationId, ORGANIZATION_JOBS_WINDOW),
      enabled: enabled && !!organizationId,
      queryFn: async () =>
        (
          await repository().getByOrganizationId(organizationId!, ORGANIZATION_JOBS_WINDOW)
        ).unwrap(),
      staleTime: 10_000,
      refetchInterval: query => (hasActive(query.state.data?.items ?? []) ? 5_000 : false),
    }),

  historyOf: (pipelineId: string, enabled: boolean) =>
    queryOptions({
      queryKey: JOBS_QUERY_KEY(pipelineId),
      enabled,
      queryFn: async () => {
        const list = await repository().getByPipelineId(pipelineId, {
          page: 1,
          pageSize: MAX_JOBS_PER_PIPELINE,
        });
        return { pipelineId, jobs: list.unwrap().items };
      },
      staleTime: 0,
      refetchInterval: query => (hasActive(query.state.data?.jobs ?? []) ? 2000 : false),
    }),
};

/** The window, its outcome mix, and whether it is partial. */
export const asJobFeed = (data: PaginatedList<JobEntity> | undefined) => {
  const jobs = data?.items ?? [];
  const totalCount = data?.pagination.totalCount ?? 0;

  return {
    jobs,
    summary: summarizeJobs(jobs),
    totalCount,
    isPartialWindow: totalCount > jobs.length,
  };
};

export const jobMutations = {
  /** Invalidates the pipeline's list, or every job list when the pipeline is not known. */
  remove: (pipelineId?: string) =>
    mutationOptions({
      mutationFn: async (jobId: string) => (await repository().deleteById(jobId)).unwrap(),
      onSuccess: () => {
        void getQueryClient().invalidateQueries({
          queryKey: pipelineId ? JOBS_QUERY_KEY(pipelineId) : JOBS_QUERY_ROOT,
        });
      },
    }),
};
