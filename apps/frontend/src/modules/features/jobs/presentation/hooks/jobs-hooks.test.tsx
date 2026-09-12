import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { ReactNode } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { renderHook, waitFor } from '@testing-library/react';
import { DependenciesProvider } from '@platform/di';
import { usePermissionsStore, PermissionScope } from '@platform/authz';
import { ScyllaResult, ScyllaError } from '@shared/utils/scylla-result.ts';
import { useJob } from './use-job';
import { useJobLogs } from './use-job-logs';
import { useDeleteJobs } from './use-delete-jobs';
import { useJobsByPipelines } from './use-jobs-by-pipelines';
import { usePipelinesJobs } from './use-pipelines-jobs';
import { useOrganizationJobs } from './use-organization-jobs';
import type { JobsRepository } from '@/modules/features/jobs/domain/repository/jobs.repository.ts';
import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';

const job = (overrides: Partial<JobEntity> = {}): JobEntity => ({
  id: 'job-1',
  pipelineId: 'pipeline-1',
  status: 'completed',
  nodeExecutions: [],
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:01:00.000Z',
  ...overrides,
});

const paginated = <T,>(items: T[]) => ({
  items,
  pagination: { totalCount: items.length, page: 1, pageSize: 100, totalPages: 1, hasNext: false, hasPrevious: false },
});

const makeFakeRepository = (overrides: Partial<JobsRepository> = {}) => {
  const getByPipelineId =
    overrides.getByPipelineId ?? vi.fn().mockResolvedValue(ScyllaResult.success(paginated([job()])));
  const getByOrganizationId =
    overrides.getByOrganizationId ?? vi.fn().mockResolvedValue(ScyllaResult.success(paginated([job()])));
  const getById = overrides.getById ?? vi.fn().mockResolvedValue(ScyllaResult.success(job()));
  const deleteById = overrides.deleteById ?? vi.fn().mockResolvedValue(ScyllaResult.success(undefined));
  const getLogs = overrides.getLogs ?? vi.fn().mockResolvedValue(ScyllaResult.success(paginated([])));
  const tailLogs = overrides.tailLogs ?? vi.fn();

  const repository: JobsRepository = {
    getByPipelineId,
    getByOrganizationId,
    getById,
    deleteById,
    getLogs,
    tailLogs,
  };
  return { repository, getByPipelineId, getByOrganizationId, getById, deleteById, getLogs, tailLogs };
};

const wrapperFor = (repository: JobsRepository) => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const Wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      <DependenciesProvider registry={{ jobs: { jobsRepository: repository } }}>
        {children}
      </DependenciesProvider>
    </QueryClientProvider>
  );
  return { Wrapper, queryClient };
};

beforeEach(() => {
  usePermissionsStore.setState({ permissions: null });
});

describe('useJob', () => {
  it('fetches a job by id', async () => {
    const { repository, getById } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useJob('job-1'), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.job).toEqual(job()));
    expect(getById).toHaveBeenCalledWith('job-1');
  });

  it('does not query for an empty jobId', () => {
    const { repository, getById } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    renderHook(() => useJob(''), { wrapper: Wrapper });
    expect(getById).not.toHaveBeenCalled();
  });

  it('falls back to a generic error message when the error is not an Error instance', async () => {
    const getById = vi.fn().mockRejectedValue('not-an-error-instance');
    const { repository } = makeFakeRepository({ getById });
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useJob('job-1'), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.errorMessage).toBe('Une erreur est survenue');
  });

  it('surfaces the real error message when the error is an Error/ScyllaError', async () => {
    const getById = vi.fn().mockResolvedValue(ScyllaResult.error(new ScyllaError('job vanished')));
    const { repository } = makeFakeRepository({ getById });
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useJob('job-1'), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.errorMessage).toBe('job vanished');
  });
});

describe('useJobLogs', () => {
  it('scopes the query to the job and, when given, the node', async () => {
    const { repository, getLogs } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    renderHook(() => useJobLogs('job-1', 'checkout'), { wrapper: Wrapper });

    await waitFor(() => expect(getLogs).toHaveBeenCalledWith('job-1', 'checkout'));
  });

  it('returns the paginated log list once loaded', async () => {
    const logs = paginated([
      { id: '1', jobId: 'job-1', nodeId: '', stream: 'stdout', line: 'hello', timestamp: 't' },
    ]);
    const { repository } = makeFakeRepository({
      getLogs: vi.fn().mockResolvedValue(ScyllaResult.success(logs)),
    });
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useJobLogs('job-1'), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.logs).toEqual(logs));
    expect(result.current.isError).toBe(false);
  });
});

describe('useDeleteJobs', () => {
  it('deletes by job id', async () => {
    const { repository, deleteById } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useDeleteJobs('pipeline-1'), { wrapper: Wrapper });

    await result.current.mutateAsync('job-1');
    expect(deleteById).toHaveBeenCalledWith('job-1');
  });

  it('invalidates that pipeline\'s jobs list on success, when a pipelineId was given', async () => {
    const { repository } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useDeleteJobs('pipeline-1'), { wrapper: Wrapper });

    await result.current.mutateAsync('job-1');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['jobs', 'pipeline', 'pipeline-1'] });
  });

  it('invalidates nothing when no pipelineId was given (caller must invalidate itself)', async () => {
    const { repository } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useDeleteJobs(), { wrapper: Wrapper });

    await result.current.mutateAsync('job-1');
    expect(invalidateSpy).not.toHaveBeenCalled();
  });
});

describe('useJobsByPipelines', () => {
  const grantListJobs = () =>
    usePermissionsStore.setState({
      permissions: {
        scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }],
      },
    });

  it('never queries when the caller lacks LIST_JOBS_BY_PIPELINE (permissions known but denied)', () => {
    usePermissionsStore.setState({ permissions: { scopes: [] } });
    const { repository, getByPipelineId } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useJobsByPipelines(['pipeline-1']), { wrapper: Wrapper });

    expect(getByPipelineId).not.toHaveBeenCalled();
    expect(result.current.canListJobs).toBe(false);
  });

  it('reports loading (not denied) while permissions are still unknown', () => {
    const { repository } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useJobsByPipelines(['pipeline-1']), { wrapper: Wrapper });

    expect(result.current.isJobsLoading).toBe(true);
  });

  it('fetches each pipeline\'s recent jobs in parallel once permitted, keyed by pipeline id', async () => {
    grantListJobs();
    const { repository, getByPipelineId } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useJobsByPipelines(['pipeline-1', 'pipeline-2']), {
      wrapper: Wrapper,
    });

    await waitFor(() => expect(result.current.jobsByPipelineId.get('pipeline-1')).toEqual([job()]));
    expect(result.current.jobsByPipelineId.get('pipeline-2')).toEqual([job()]);
    expect(getByPipelineId).toHaveBeenCalledWith('pipeline-1', { page: 1, pageSize: 10 });
    expect(getByPipelineId).toHaveBeenCalledWith('pipeline-2', { page: 1, pageSize: 10 });
  });
});

describe('usePipelinesJobs', () => {
  it('lists a pipeline\'s jobs, paginated', async () => {
    const { repository, getByPipelineId } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => usePipelinesJobs('pipeline-1'), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.jobs).toEqual([job()]));
    expect(getByPipelineId).toHaveBeenCalledWith('pipeline-1', { page: 1, pageSize: 10 });
  });

  it('does not query for an empty pipelineId', () => {
    const { repository, getByPipelineId } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    renderHook(() => usePipelinesJobs(''), { wrapper: Wrapper });
    expect(getByPipelineId).not.toHaveBeenCalled();
  });

  it('falls back to a generic error message on a non-Error rejection', async () => {
    const { repository } = makeFakeRepository({ getByPipelineId: vi.fn().mockRejectedValue('boom') });
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => usePipelinesJobs('pipeline-1'), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.errorMessage).toBe('Une erreur est survenue');
  });
});

describe('useOrganizationJobs', () => {
  it('lists the organization\'s recent jobs and summarizes them', async () => {
    const { repository } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useOrganizationJobs('org-1'), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.jobs).toEqual([job()]));
    expect(result.current.summary.completed).toBe(1);
    expect(result.current.isPartialWindow).toBe(false);
  });

  it('does not query for a null organizationId', () => {
    const { repository, getByOrganizationId } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    renderHook(() => useOrganizationJobs(null), { wrapper: Wrapper });
    expect(getByOrganizationId).not.toHaveBeenCalled();
  });

  it('does not query when explicitly disabled, even with a valid organizationId', () => {
    const { repository, getByOrganizationId } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    renderHook(() => useOrganizationJobs('org-1', false), { wrapper: Wrapper });
    expect(getByOrganizationId).not.toHaveBeenCalled();
  });

  it('flags a partial window when more jobs exist than the page fetched', async () => {
    const partial = {
      items: [job()],
      pagination: { totalCount: 50, page: 1, pageSize: 100, totalPages: 1, hasNext: false, hasPrevious: false },
    };
    const { repository } = makeFakeRepository({
      getByOrganizationId: vi.fn().mockResolvedValue(ScyllaResult.success(partial)),
    });
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useOrganizationJobs('org-1'), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.isPartialWindow).toBe(true));
    expect(result.current.totalCount).toBe(50);
  });
});
