// @vitest-environment node
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { QueryClient } from '@tanstack/query-core';
import { setDependencyRegistry } from '@platform/di';
import { setQueryClient } from '@platform/query';
import { runMutationFn, runOnSuccess, runQueryFn } from '@/test/queries.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import type { JobEntity } from '../../domain/entities/job.entity.ts';
import type { JobsRepository } from '../../domain/repository/jobs.repository.ts';
import { JOBS_QUERY_KEY, JOBS_QUERY_ROOT, JOB_QUERY_KEY } from '../jobs.query-keys.ts';
import { asJobFeed, jobMutations, jobQueries } from '../jobs.queries.ts';

const job = (overrides: Partial<JobEntity> = {}): JobEntity => ({
  id: 'job-1',
  pipelineId: 'pipeline-1',
  status: 'completed',
  nodeExecutions: [],
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:01:00.000Z',
  ...overrides,
});

const page = (items: JobEntity[], totalCount = items.length): PaginatedList<JobEntity> => ({
  items,
  pagination: {
    totalCount,
    page: 1,
    pageSize: 20,
    totalPages: 1,
    hasNext: false,
    hasPrevious: false,
  },
});

/** What `refetchInterval` is handed — only `state.data` is ever read. */
const withData = <TData>(data: TData | undefined) =>
  ({ state: { data } }) as never;

let repository: JobsRepository;
let invalidate: ReturnType<typeof vi.fn>;

beforeEach(() => {
  repository = {
    getByPipelineId: vi.fn().mockResolvedValue(ScyllaResult.success(page([job()]))),
    getByOrganizationId: vi.fn().mockResolvedValue(ScyllaResult.success(page([job()]))),
    getById: vi.fn().mockResolvedValue(ScyllaResult.success(job())),
    deleteById: vi.fn().mockResolvedValue(ScyllaResult.success(undefined)),
    getLogs: vi.fn(),
    tailLogs: vi.fn(),
  };

  setDependencyRegistry({ jobs: { jobsRepository: repository } });

  const queryClient = new QueryClient();
  invalidate = vi.fn();
  queryClient.invalidateQueries = invalidate as unknown as QueryClient['invalidateQueries'];
  setQueryClient(queryClient);
});

afterEach(() => {
  setDependencyRegistry(null);
  setQueryClient(null);
});

describe('jobQueries.byId', () => {
  it('reads one job, under the key the listings share a root with', async () => {
    const options = jobQueries.byId('job-1');

    expect(options.queryKey).toEqual(JOB_QUERY_KEY('job-1'));
    await expect(runQueryFn(options)).resolves.toEqual(job());
    expect(repository.getById).toHaveBeenCalledWith('job-1');
  });

  it('stays idle for an empty jobId rather than asking for nothing', () => {
    expect(jobQueries.byId('').enabled).toBe(false);
  });

  it('keeps polling while the job is unfinished, and stops once it is', () => {
    const interval = jobQueries.byId('job-1').refetchInterval as (query: never) => number | false;

    expect(interval(withData(job({ status: 'running' })))).toBe(3000);
    expect(interval(withData(job({ status: 'pending' })))).toBe(3000);
    expect(interval(withData(job({ status: 'completed' })))).toBe(false);
    expect(interval(withData(job({ status: 'failed' })))).toBe(false);
  });

  it('does not poll before the first answer, when there is no status to judge', () => {
    const interval = jobQueries.byId('job-1').refetchInterval as (query: never) => number | false;
    expect(interval(withData(undefined))).toBe(false);
  });
});

describe('jobQueries.byPipeline', () => {
  const pagination = { page: 2, pageSize: 25 };

  it('carries the pagination in the key, so two pages are two cache entries', () => {
    const first = jobQueries.byPipeline('pipeline-1', { page: 1, pageSize: 25 });
    const second = jobQueries.byPipeline('pipeline-1', pagination);

    expect(first.queryKey).not.toEqual(second.queryKey);
    expect(second.queryKey.slice(0, 3)).toEqual([...JOBS_QUERY_KEY('pipeline-1')]);
  });

  it('asks the repository for exactly the page it was given', async () => {
    await runQueryFn(jobQueries.byPipeline('pipeline-1', pagination));
    expect(repository.getByPipelineId).toHaveBeenCalledWith('pipeline-1', pagination);
  });

  it('stays idle until the caller says the page size is known', () => {
    const options = jobQueries.byPipeline('pipeline-1', pagination, { enabled: false });
    expect(options.enabled).toBe(false);
  });

  it('stays idle for an empty pipelineId even when the caller is ready', () => {
    expect(jobQueries.byPipeline('', pagination, { enabled: true }).enabled).toBe(false);
  });

  it('polls only while something in the page is still running', () => {
    const interval = jobQueries.byPipeline('pipeline-1', pagination).refetchInterval as (
      query: never,
    ) => number | false;

    expect(interval(withData(page([job({ status: 'running' })])))).toBe(5000);
    expect(interval(withData(page([job({ status: 'pending' })])))).toBe(5000);
    expect(interval(withData(page([job({ status: 'completed' })])))).toBe(false);
  });
});

describe('jobQueries.byOrganization', () => {
  it('reads one fixed window of recent runs', async () => {
    await runQueryFn(jobQueries.byOrganization('org-1'));
    expect(repository.getByOrganizationId).toHaveBeenCalledWith('org-1', {
      page: 1,
      pageSize: 100,
    });
  });

  it('stays idle for a null organizationId', () => {
    expect(jobQueries.byOrganization(null).enabled).toBe(false);
  });

  it('stays idle when the caller disables it, valid organization or not', () => {
    expect(jobQueries.byOrganization('org-1', false).enabled).toBe(false);
  });
});

describe('asJobFeed', () => {
  it('summarizes the window it was handed', () => {
    const feed = asJobFeed(page([job({ status: 'completed' }), job({ id: 'job-2', status: 'failed' })]));

    expect(feed.jobs).toHaveLength(2);
    expect(feed.summary.completed).toBe(1);
    expect(feed.summary.failed).toBe(1);
  });

  it('flags the window as partial when the server holds more than it returned', () => {
    expect(asJobFeed(page([job()], 250)).isPartialWindow).toBe(true);
  });

  it('does not flag a window that holds every run there is', () => {
    expect(asJobFeed(page([job()], 1)).isPartialWindow).toBe(false);
  });

  it('answers an empty, whole feed before anything has arrived', () => {
    const feed = asJobFeed(undefined);

    expect(feed.jobs).toEqual([]);
    expect(feed.totalCount).toBe(0);
    expect(feed.isPartialWindow).toBe(false);
  });
});

describe('jobMutations.remove', () => {
  it('deletes by job id', async () => {
    await runMutationFn(jobMutations.remove('pipeline-1'), 'job-1');
    expect(repository.deleteById).toHaveBeenCalledWith('job-1');
  });

  it("invalidates that pipeline's list when it knows which pipeline it is", () => {
    runOnSuccess(jobMutations.remove('pipeline-1'), undefined, 'job-1');
    expect(invalidate).toHaveBeenCalledWith({ queryKey: JOBS_QUERY_KEY('pipeline-1') });
  });

  it('invalidates every job listing when it does not — the details page knows no pipeline', () => {
    runOnSuccess(jobMutations.remove(), undefined, 'job-1');
    expect(invalidate).toHaveBeenCalledWith({ queryKey: JOBS_QUERY_ROOT });
  });
});
