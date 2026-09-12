import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { ReactNode } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, renderHook, waitFor } from '@testing-library/react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { DependenciesProvider } from '@platform/di';
import { useContextStore } from '@platform/context';
import { ScyllaResult, ScyllaError } from '@shared/utils/scylla-result.ts';
import { useCreatePipeline } from './use-create-pipeline';
import { useDeletePipeline } from './use-delete-pipeline';
import { useDuplicatePipeline } from './use-duplicate-pipeline';
import { useOrganizationPipelines } from './use-organization-pipelines';
import { usePipelinesMetadata } from './use-pipelines-metadata';
import { usePipeline } from './use-pipeline';
import { useRunPipeline } from './use-run-pipeline';
import { useUpdatePipeline } from './use-update-pipeline';
import type { PipelineRepository } from '@/modules/features/pipeline/domain/repository/pipeline.repository.ts';
import type { PipelineEntity } from '@/modules/features/pipeline/domain/entities/pipeline.entity.ts';
import type { PipelineMetadata } from '@/modules/features/pipeline/domain/structs/pipeline.struct.ts';
import type * as AgentsModule from '@/modules/features/agents';

type AgentEntity = AgentsModule.AgentEntity;

const navigateMock = vi.fn();
vi.mock('react-router-dom', () => ({
  useNavigate: () => navigateMock,
  useLocation: () => ({ pathname: '/acme/projects/project-1' }),
}));

const toastSuccess = vi.fn();
const toastWarning = vi.fn();
vi.mock('sonner', () => ({
  toast: {
    success: (...args: unknown[]) => toastSuccess(...args),
    warning: (...args: unknown[]) => toastWarning(...args),
  },
}));

let agentsFixture: AgentEntity[] = [];
let canListAgentsFixture = true;
vi.mock('@/modules/features/agents', async importOriginal => {
  const actual = await importOriginal<typeof AgentsModule>();
  return {
    ...actual,
    useAgents: () => ({ agents: agentsFixture, canListAgents: canListAgentsFixture }),
  };
});

const pipeline = (overrides: Partial<PipelineEntity> = {}): PipelineEntity => ({
  id: 'pipeline-1',
  projectId: 'project-1',
  name: 'ci',
  nodes: [],
  ...overrides,
});

const metadata = (overrides: Partial<PipelineMetadata> = {}): PipelineMetadata => ({
  id: 'pipeline-1',
  projectId: 'project-1',
  name: 'ci',
  nodeCount: 2,
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

const agent = (overrides: Partial<AgentEntity> = {}): AgentEntity => ({
  id: 'agent-1',
  organizationId: 'org-1',
  name: 'runner-1',
  isActive: true,
  connected: true,
  lastSeen: '2026-01-01T00:00:00.000Z',
  inFlight: 0,
  host: null,
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

const paginated = <T,>(items: T[], totalCount = items.length) => ({
  items,
  pagination: { totalCount, page: 1, pageSize: 100, totalPages: 1, hasNext: false, hasPrevious: false },
});

const makeFakeRepository = (overrides: Partial<PipelineRepository> = {}) => {
  const getMetadataByProjectId =
    overrides.getMetadataByProjectId ??
    vi.fn().mockResolvedValue(ScyllaResult.success(paginated([metadata()])));
  const getMetadataByOrganizationId =
    overrides.getMetadataByOrganizationId ??
    vi.fn().mockResolvedValue(ScyllaResult.success(paginated([metadata()])));
  const deleteById = overrides.deleteById ?? vi.fn().mockResolvedValue(ScyllaResult.success(undefined));
  const run = overrides.run ?? vi.fn().mockResolvedValue(ScyllaResult.success(undefined));
  const create = overrides.create ?? vi.fn().mockResolvedValue(ScyllaResult.success(undefined));
  const getById = overrides.getById ?? vi.fn().mockResolvedValue(ScyllaResult.success(pipeline()));
  const edit = overrides.edit ?? vi.fn().mockResolvedValue(ScyllaResult.success(pipeline()));

  const repository: PipelineRepository = {
    getMetadataByProjectId,
    getMetadataByOrganizationId,
    deleteById,
    run,
    create,
    getById,
    edit,
  };
  return {
    repository,
    getMetadataByProjectId,
    getMetadataByOrganizationId,
    deleteById,
    run,
    create,
    getById,
    edit,
  };
};

const wrapperFor = (repository: PipelineRepository) => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const Wrapper = ({ children }: { children: ReactNode }) => (
    <I18nProvider i18n={i18n}>
      <QueryClientProvider client={queryClient}>
        <DependenciesProvider registry={{ pipeline: { pipelineRepository: repository } }}>
          {children}
        </DependenciesProvider>
      </QueryClientProvider>
    </I18nProvider>
  );
  return { Wrapper, queryClient };
};

beforeEach(() => {
  navigateMock.mockClear();
  toastSuccess.mockClear();
  toastWarning.mockClear();
  agentsFixture = [];
  canListAgentsFixture = true;
  useContextStore.setState({
    organization: { id: null, name: null },
    project: { id: null, name: null },
    pipeline: { id: null, name: null },
  });
});

describe('useCreatePipeline', () => {
  it('creates the pipeline, invalidates that project\'s list, and navigates there', async () => {
    useContextStore.setState({ project: { id: 'project-1', name: 'Acme' } });
    const { repository, create } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useCreatePipeline(), { wrapper: Wrapper });

    // onSuccess calls goToProject, which sets the (real) context store -
    // wrap so that re-render lands inside act().
    await act(() => result.current.mutateAsync({ projectId: 'project-1', name: 'ci', nodes: [] }));

    expect(create).toHaveBeenCalledWith({ projectId: 'project-1', name: 'ci', nodes: [] });
    expect(toastSuccess).toHaveBeenCalledWith('Pipeline created');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['pipelines', 'project-1'] });
    expect(navigateMock).toHaveBeenCalled();
  });

  it('does not invalidate or navigate when there is no active project in context', async () => {
    const { repository } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useCreatePipeline(), { wrapper: Wrapper });

    await result.current.mutateAsync({ projectId: 'project-1', name: 'ci', nodes: [] });

    expect(toastSuccess).toHaveBeenCalledWith('Pipeline created');
    expect(invalidateSpy).not.toHaveBeenCalled();
    expect(navigateMock).not.toHaveBeenCalled();
  });
});

describe('useDeletePipeline', () => {
  it('deletes by id, toasts, and invalidates the whole pipelines prefix', async () => {
    const { repository, deleteById } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useDeletePipeline(), { wrapper: Wrapper });

    await result.current.mutateAsync('pipeline-1');

    expect(deleteById).toHaveBeenCalledWith('pipeline-1');
    expect(toastSuccess).toHaveBeenCalledWith('Pipeline deleted');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['pipelines'] });
  });
});

describe('useDuplicatePipeline', () => {
  it('reads the pipeline then creates a copy named "<name> (copy)"', async () => {
    useContextStore.setState({ project: { id: 'project-1', name: 'Acme' } });
    const { repository, getById, create } = makeFakeRepository({
      getById: vi.fn().mockResolvedValue(ScyllaResult.success(pipeline({ name: 'ci', nodes: [] }))),
    });
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useDuplicatePipeline(), { wrapper: Wrapper });

    // onSuccess calls goToProject, which sets the (real) context store.
    await act(() => result.current.mutateAsync('pipeline-1'));

    expect(getById).toHaveBeenCalledWith('pipeline-1');
    expect(create).toHaveBeenCalledWith({
      name: 'ci (copy)',
      projectId: 'project-1',
      nodes: [],
    });
    expect(toastSuccess).toHaveBeenCalledWith('Pipeline duplicated successfully');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['pipelines'] });
    expect(navigateMock).toHaveBeenCalled();
  });

  it('propagates a failure from the read step without ever calling create', async () => {
    const error = new ScyllaError('not found');
    const { repository, create } = makeFakeRepository({
      getById: vi.fn().mockResolvedValue(ScyllaResult.error(error)),
    });
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useDuplicatePipeline(), { wrapper: Wrapper });

    await expect(result.current.mutateAsync('pipeline-1')).rejects.toBe(error);
    expect(create).not.toHaveBeenCalled();
  });
});

describe('useOrganizationPipelines', () => {
  it('lists the organization\'s pipelines, flagging a partial window', async () => {
    const { repository } = makeFakeRepository({
      getMetadataByOrganizationId: vi
        .fn()
        .mockResolvedValue(ScyllaResult.success(paginated([metadata()], 50))),
    });
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useOrganizationPipelines('org-1'), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.pipelines).toEqual([metadata()]));
    expect(result.current.isPartialWindow).toBe(true);
    expect(result.current.totalCount).toBe(50);
  });

  it('does not query for a null organizationId', () => {
    const { repository, getMetadataByOrganizationId } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    renderHook(() => useOrganizationPipelines(null), { wrapper: Wrapper });
    expect(getMetadataByOrganizationId).not.toHaveBeenCalled();
  });
});

describe('usePipelinesMetadata', () => {
  it('lists a project\'s pipelines, paginated', async () => {
    const { repository, getMetadataByProjectId } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => usePipelinesMetadata('project-1'), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.pipelines?.items).toEqual([metadata()]));
    expect(getMetadataByProjectId).toHaveBeenCalledWith('project-1', {
      page: 1,
      pageSize: 10,
    });
  });

  it('falls back to a generic error message on a non-Error rejection', async () => {
    const { repository } = makeFakeRepository({
      getMetadataByProjectId: vi.fn().mockRejectedValue('boom'),
    });
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => usePipelinesMetadata('project-1'), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.errorMessage).toBe('Une erreur est survenue');
  });
});

describe('usePipeline', () => {
  it('fetches by id', async () => {
    const { repository, getById } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => usePipeline('pipeline-1'), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.pipeline).toEqual(pipeline()));
    expect(getById).toHaveBeenCalledWith('pipeline-1');
  });

  it('surfaces a repository error', async () => {
    const error = new ScyllaError('boom');
    const { repository } = makeFakeRepository({
      getById: vi.fn().mockResolvedValue(ScyllaResult.error(error)),
    });
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => usePipeline('pipeline-1'), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error).toBe(error);
  });
});

describe('useRunPipeline', () => {
  it('invalidates that pipeline\'s jobs regardless of the agent-connectivity outcome', async () => {
    const { repository, run } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useRunPipeline(), { wrapper: Wrapper });

    await result.current.mutateAsync('pipeline-1');

    expect(run).toHaveBeenCalledWith('pipeline-1');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['jobs', 'pipeline', 'pipeline-1'] });
  });

  it('without LIST_AGENTS, tells the caller to check for themselves rather than claim none is connected', async () => {
    canListAgentsFixture = false;
    const { repository } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useRunPipeline(), { wrapper: Wrapper });

    await result.current.mutateAsync('pipeline-1');

    expect(toastSuccess).toHaveBeenCalledWith('Pipeline ran — check that your agents are connected');
    expect(toastWarning).not.toHaveBeenCalled();
  });

  it('warns (with a link to Agents) when permitted to look and no agent is connected', async () => {
    canListAgentsFixture = true;
    agentsFixture = [agent({ connected: false })];
    useContextStore.setState({ organization: { id: 'org-1', name: 'Acme Corp' } });
    const { repository } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useRunPipeline(), { wrapper: Wrapper });

    await result.current.mutateAsync('pipeline-1');

    expect(toastWarning).toHaveBeenCalledWith(
      'Job queued — no agent connected',
      expect.objectContaining({ action: expect.objectContaining({ label: 'Agents' }) }),
    );
    expect(toastSuccess).not.toHaveBeenCalled();

    // The action navigates to Agents under the current org's slug.
    const [, options] = toastWarning.mock.calls[0] as [string, { action: { onClick: () => void } }];
    options.action.onClick();
    expect(navigateMock).toHaveBeenCalledWith('/acme-corp/agents');
  });

  it('reports a plain success once at least one agent is connected', async () => {
    canListAgentsFixture = true;
    agentsFixture = [agent({ connected: false }), agent({ id: 'agent-2', connected: true })];
    const { repository } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useRunPipeline(), { wrapper: Wrapper });

    await result.current.mutateAsync('pipeline-1');

    expect(toastSuccess).toHaveBeenCalledWith('Pipeline ran');
    expect(toastWarning).not.toHaveBeenCalled();
  });
});

describe('useUpdatePipeline', () => {
  it('edits by id, invalidates the project list and the single-pipeline cache, toasts, and navigates', async () => {
    useContextStore.setState({ project: { id: 'project-1', name: 'Acme' } });
    const { repository, edit } = makeFakeRepository({
      edit: vi
        .fn()
        .mockResolvedValue(ScyllaResult.success(pipeline({ id: 'pipeline-1', projectId: 'project-1' }))),
    });
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useUpdatePipeline(), { wrapper: Wrapper });

    // onSuccess calls goToProject, which sets the (real) context store.
    await act(() => result.current.mutateAsync({ id: 'pipeline-1', nodes: [] }));

    expect(edit).toHaveBeenCalledWith('pipeline-1', [], undefined);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['pipelines', 'project-1'] });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['pipeline', 'pipeline-1'] });
    expect(toastSuccess).toHaveBeenCalledWith('Pipeline edited');
    expect(navigateMock).toHaveBeenCalled();
  });

  it('does not navigate when there is no active project in context', async () => {
    const { repository } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useUpdatePipeline(), { wrapper: Wrapper });

    await result.current.mutateAsync({ id: 'pipeline-1', nodes: [] });

    expect(toastSuccess).toHaveBeenCalledWith('Pipeline edited');
    expect(navigateMock).not.toHaveBeenCalled();
  });
});
