// @vitest-environment node
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { setDependencyRegistry } from '@platform/di';
import { Permission, PermissionScope, usePermissionsStore } from '@platform/authz';
import { useContextStore } from '@platform/context';
import { runQueryFn } from '@/test/queries.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { JobEntity } from '../domain/entities/job.entity.ts';
import type { JobsRepository } from '../domain/repository/jobs.repository.ts';
import { jobsByPipelinesQueries } from './jobs-by-pipelines.queries.ts';

const job = (id: string): JobEntity => ({
  id,
  pipelineId: 'pipeline-1',
  status: 'completed',
  nodeExecutions: [],
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:01:00.000Z',
});

const listOf = (items: JobEntity[]) => ({
  items,
  pagination: {
    totalCount: items.length,
    page: 1,
    pageSize: 10,
    totalPages: 1,
    hasNext: false,
    hasPrevious: false,
  },
});

/** Drives the real permissions store — never a mocked `can`. */
const grantEverything = () =>
  usePermissionsStore.setState({
    permissions: {
      scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }],
    },
  });

const grantNothing = () => usePermissionsStore.setState({ permissions: { scopes: [] } });

let repository: JobsRepository;

beforeEach(() => {
  repository = {
    getByPipelineId: vi.fn().mockResolvedValue(ScyllaResult.success(listOf([job('job-1')]))),
    getByOrganizationId: vi.fn(),
    getById: vi.fn(),
    deleteById: vi.fn(),
    getLogs: vi.fn(),
    tailLogs: vi.fn(),
  };
  setDependencyRegistry({ jobs: { jobsRepository: repository } });
});

afterEach(() => {
  setDependencyRegistry(null);
  usePermissionsStore.setState({ permissions: null });
});

describe('jobsByPipelinesQueries', () => {
  it('checks LIST_JOBS_BY_PIPELINE for itself, because the consumer entered on its own permission', () => {
    grantNothing();
    const { canListJobs, queries } = jobsByPipelinesQueries(['pipeline-1', 'pipeline-2']);

    expect(canListJobs).toBe(false);
    // Still one entry per pipeline — the caller needs the shape — but none of
    // them will ask, which is what stops a denial toast per row.
    expect(queries).toHaveLength(2);
    expect(queries.every(query => query.enabled === false)).toBe(true);
  });

  it('reports loading, not denied, while the permissions are still unknown', () => {
    usePermissionsStore.setState({ permissions: null });
    const { canListJobs, combine } = jobsByPipelinesQueries(['pipeline-1']);

    expect(canListJobs).toBe(false);
    expect(combine([{ isLoading: false, isError: false }]).isJobsLoading).toBe(true);
  });

  it('asks for each pipeline once permitted, capped at a history strip rather than the whole log', async () => {
    grantEverything();
    const { queries } = jobsByPipelinesQueries(['pipeline-1']);

    expect(queries[0].enabled).toBe(true);
    await expect(runQueryFn(queries[0])).resolves.toEqual({
      pipelineId: 'pipeline-1',
      jobs: [job('job-1')],
    });
    expect(repository.getByPipelineId).toHaveBeenCalledWith('pipeline-1', {
      page: 1,
      pageSize: 10,
    });
  });

  it('keys the combined map by pipeline id, skipping the ones that answered nothing', () => {
    grantEverything();
    const { combine } = jobsByPipelinesQueries(['pipeline-1', 'pipeline-2']);

    const { jobsByPipelineId } = combine([
      { data: { pipelineId: 'pipeline-1', jobs: [job('job-1')] }, isLoading: false, isError: false },
      { isLoading: false, isError: false },
    ]);

    expect([...jobsByPipelineId.keys()]).toEqual(['pipeline-1']);
    expect(jobsByPipelineId.get('pipeline-1')).toEqual([job('job-1')]);
  });

  it('reports the fan-out as failed when any one pipeline failed', () => {
    grantEverything();
    const { combine } = jobsByPipelinesQueries(['pipeline-1', 'pipeline-2']);

    expect(
      combine([
        { isLoading: false, isError: false },
        { isLoading: false, isError: true },
      ]).isJobsError,
    ).toBe(true);
  });

  it('resolves against the project the page is already scoped to, with no target passed', () => {
    // The ambient target is the context store's project — which is the whole
    // reason `can()` is called without one here.
    useContextStore.setState({
      organization: { id: 'org-1', name: 'Acme' },
      project: { id: 'project-1', name: 'Acme project' },
    });

    usePermissionsStore.setState({
      permissions: {
        scopes: [
          {
            scope: PermissionScope.PROJECT,
            scopeId: 'project-1',
            access: { kind: 'restricted', permissions: [Permission.LIST_JOBS_BY_PIPELINE] },
          },
        ],
      },
    });

    expect(jobsByPipelinesQueries(['pipeline-1']).canListJobs).toBe(true);
  });
});
