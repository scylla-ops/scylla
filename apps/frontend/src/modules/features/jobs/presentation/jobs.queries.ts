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

/**
 * Every read and write this module performs, declared as plain data.
 *
 * This is what the eight hooks under `presentation/hooks/` were, with the
 * framework taken out. Two of them — the organization feed and the per-pipeline
 * fan-out — are consumed through the barrel by `dashboard` and `pipeline`,
 * which are still React: a `queryOptions` object has no framework in it, so
 * `useQuery` takes the same one `createQuery` does and both halves share a
 * cache entry.
 *
 * The repository is resolved per call, never at module load: the registry is
 * installed by the composition root and swapped by tests.
 */
const repository = () => getModuleDomain<typeof JobsModule.domain>('jobs').jobsRepository;

/** Enough recent jobs to draw a pipeline's history strip, not its whole log. */
const MAX_JOBS_PER_PIPELINE = 10;

/**
 * How many recent runs the organization feed reads.
 *
 * `ListOrganizationJobs` paginates and does not aggregate, so every figure the
 * dashboard shows is computed over this window — see {@link JobsSummary}.
 */
export const ORGANIZATION_JOBS_WINDOW: PaginationParams = { page: 1, pageSize: 100 };

/** Something is still moving, so the list is worth asking for again. */
const hasActive = (jobs: readonly JobEntity[]) => jobs.some(job => isActiveStatus(job.status));

export const jobQueries = {
  /**
   * One job, polled while it runs.
   *
   * The interval stops on its own at a finished status rather than being
   * cleared by whoever opened the page: the query is the only thing that knows
   * the job is over.
   */
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

  /**
   * One page of a pipeline's jobs.
   *
   * `enabled` takes the caller's readiness because the page size is measured
   * from the layout: asking before the table area exists would fetch a page
   * sized for a container that has not been laid out yet, then immediately
   * fetch again.
   */
  byPipeline: (pipelineId: string, pagination: PaginationParams, options: { enabled?: boolean } = {}) =>
    queryOptions<PaginatedList<JobEntity>>({
      queryKey: [...JOBS_QUERY_KEY(pipelineId), pagination],
      enabled: (options.enabled ?? true) && !!pipelineId,
      queryFn: async () => (await repository().getByPipelineId(pipelineId, pagination)).unwrap(),
      staleTime: 0,
      refetchInterval: query => (hasActive(query.state.data?.items ?? []) ? 5000 : false),
    }),

  /**
   * The organization's recent runs, plus the outcome mix over that window.
   *
   * Part of the module's public API: the dashboard shows organization-wide run
   * activity, which is a jobs query and belongs here. Polls while something is
   * still running and goes quiet once the window holds only finished jobs.
   */
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

  /** The most recent jobs of one pipeline, for its history strip. */
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

/**
 * Turns a page of jobs into the shape a feed reads: the window, its outcome
 * mix, and whether it is only a window.
 *
 * A plain function over the query's data rather than part of the options: it is
 * the one piece `useOrganizationJobs` did beyond fetching, and both bindings
 * want it. `summarizeJobs` is domain and stays there.
 */
export const asJobFeed = (data: PaginatedList<JobEntity> | undefined) => {
  const jobs = data?.items ?? [];
  const totalCount = data?.pagination.totalCount ?? 0;

  return {
    jobs,
    summary: summarizeJobs(jobs),
    totalCount,
    /** True when the feed is a window over a larger history, so figures are partial. */
    isPartialWindow: totalCount > jobs.length,
  };
};

export const jobMutations = {
  /**
   * Deletes one job.
   *
   * Invalidates the pipeline's list when it knows which one it is, and the
   * whole `['jobs']` root otherwise: a job deleted from the details page is
   * also gone from the organization feed, and from the history strip of the
   * pipeline it belonged to.
   */
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
