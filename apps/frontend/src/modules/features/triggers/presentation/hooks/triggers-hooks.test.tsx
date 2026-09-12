import { describe, it, expect, vi } from 'vitest';
import type { ReactNode } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { renderHook, waitFor } from '@testing-library/react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { DependenciesProvider } from '@platform/di';
import { ScyllaResult, ScyllaError } from '@shared/utils/scylla-result.ts';
import { usePipelineTriggers, TRIGGERS_QUERY_KEY } from './use-pipeline-triggers';
import { useCreateTrigger } from './use-create-trigger';
import { useDeleteTrigger } from './use-delete-trigger';
import { useFireTriggerNow } from './use-fire-trigger-now';
import { useSetTriggerEnabled } from './use-set-trigger-enabled';
import { useUpdateTrigger } from './use-update-trigger';
import { TriggerKind } from '@/modules/features/triggers/domain/structs/trigger-source.struct.ts';
import type { TriggersRepository } from '@/modules/features/triggers/domain/repository/triggers.repository.ts';
import type { TriggerEntity, TriggerDraft } from '@/modules/features/triggers/domain/entities/trigger.entity.ts';

const toastSuccess = vi.fn();
vi.mock('sonner', () => ({
  toast: { success: (...args: unknown[]) => toastSuccess(...args) },
}));

const PIPELINE_ID = 'pipeline-1';

const trigger = (overrides: Partial<TriggerEntity> = {}): TriggerEntity => ({
  id: 'trigger-1',
  pipelineId: PIPELINE_ID,
  name: 'github-push',
  source: { kind: TriggerKind.Webhook, signatureHeader: '', webhookUrl: 'https://x/webhooks/1' },
  inputs: [],
  enabled: true,
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

const draft: TriggerDraft = {
  name: 'github-push',
  source: { kind: TriggerKind.Webhook, signatureHeader: '' },
  inputs: [],
};

const makeFakeRepository = (overrides: Partial<TriggersRepository> = {}) => {
  const listByPipelineId = vi.fn().mockResolvedValue(ScyllaResult.success([trigger()]));
  const getById = vi.fn().mockResolvedValue(ScyllaResult.success(trigger()));
  const create = vi.fn().mockResolvedValue(ScyllaResult.success({ trigger: trigger() }));
  const update = vi.fn().mockResolvedValue(ScyllaResult.success(trigger()));
  const deleteById = vi.fn().mockResolvedValue(ScyllaResult.success(undefined));
  const setEnabled = vi
    .fn()
    .mockImplementation((id: string, enabled: boolean) =>
      Promise.resolve(ScyllaResult.success(trigger({ id, enabled }))),
    );
  const fireNow = vi.fn().mockResolvedValue(ScyllaResult.success('job-1'));

  const repository: TriggersRepository = {
    listByPipelineId,
    getById,
    create,
    update,
    deleteById,
    setEnabled,
    fireNow,
    ...overrides,
  };
  return { repository, listByPipelineId, getById, create, update, deleteById, setEnabled, fireNow };
};

const wrapperFor = (repository: TriggersRepository) => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const Wrapper = ({ children }: { children: ReactNode }) => (
    <I18nProvider i18n={i18n}>
      <QueryClientProvider client={queryClient}>
        <DependenciesProvider registry={{ triggers: { triggersRepository: repository } }}>
          {children}
        </DependenciesProvider>
      </QueryClientProvider>
    </I18nProvider>
  );
  return { Wrapper, queryClient };
};

describe('usePipelineTriggers', () => {
  it('lists a pipeline\'s triggers', async () => {
    const { repository, listByPipelineId } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => usePipelineTriggers(PIPELINE_ID), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.triggers).toHaveLength(1));
    expect(listByPipelineId).toHaveBeenCalledWith(PIPELINE_ID);
  });

  it('does not fetch when pipelineId is empty', () => {
    const { repository, listByPipelineId } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    renderHook(() => usePipelineTriggers(''), { wrapper: Wrapper });
    expect(listByPipelineId).not.toHaveBeenCalled();
  });

  it('surfaces a repository error', async () => {
    const error = new ScyllaError('boom', { cause: { code: 'INTERNAL' } });
    const { repository } = makeFakeRepository({
      listByPipelineId: vi.fn().mockResolvedValue(ScyllaResult.error(error)),
    });
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => usePipelineTriggers(PIPELINE_ID), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error).toBe(error);
  });
});

describe('useCreateTrigger', () => {
  it('creates a trigger for the pipeline, toasts, and invalidates the list', async () => {
    const { repository, create } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useCreateTrigger(PIPELINE_ID), { wrapper: Wrapper });

    const created = await result.current.mutateAsync(draft);

    expect(create).toHaveBeenCalledWith(PIPELINE_ID, draft);
    expect(created.trigger.id).toBe('trigger-1');
    expect(toastSuccess).toHaveBeenCalledWith('Trigger created');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: TRIGGERS_QUERY_KEY(PIPELINE_ID) });
  });
});

describe('useUpdateTrigger', () => {
  it('updates by id and invalidates the list', async () => {
    const { repository, update } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useUpdateTrigger(PIPELINE_ID), { wrapper: Wrapper });

    await result.current.mutateAsync({ triggerId: 'trigger-1', draft });

    expect(update).toHaveBeenCalledWith('trigger-1', draft);
    expect(toastSuccess).toHaveBeenCalledWith('Trigger updated');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: TRIGGERS_QUERY_KEY(PIPELINE_ID) });
  });
});

describe('useDeleteTrigger', () => {
  it('deletes by id and invalidates the list', async () => {
    const { repository, deleteById } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useDeleteTrigger(PIPELINE_ID), { wrapper: Wrapper });

    await result.current.mutateAsync('trigger-1');

    expect(deleteById).toHaveBeenCalledWith('trigger-1');
    expect(toastSuccess).toHaveBeenCalledWith('Trigger deleted');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: TRIGGERS_QUERY_KEY(PIPELINE_ID) });
  });
});

describe('useFireTriggerNow', () => {
  it('fires the trigger and invalidates both the jobs and triggers queries', async () => {
    const { repository, fireNow } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useFireTriggerNow(PIPELINE_ID), { wrapper: Wrapper });

    const jobId = await result.current.mutateAsync('trigger-1');

    expect(fireNow).toHaveBeenCalledWith('trigger-1');
    expect(jobId).toBe('job-1');
    expect(toastSuccess).toHaveBeenCalledWith('Trigger fired — run started');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['jobs', 'pipeline', PIPELINE_ID] });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: TRIGGERS_QUERY_KEY(PIPELINE_ID) });
  });
});

describe('useSetTriggerEnabled', () => {
  it('optimistically toggles the cached list before the call resolves', async () => {
    // A manually-controlled promise so the repository call stays pending until
    // this test says so - proves the cache flips *before* the network settles,
    // not just that it eventually matches.
    let resolveCall: (v: ScyllaResult<TriggerEntity>) => void;
    const pending = new Promise<ScyllaResult<TriggerEntity>>(resolve => {
      resolveCall = resolve;
    });
    const setEnabled = vi.fn().mockReturnValue(pending);
    const { repository } = makeFakeRepository({ setEnabled });
    const { Wrapper, queryClient } = wrapperFor(repository);

    // Seed the cache the way usePipelineTriggers would have.
    queryClient.setQueryData(TRIGGERS_QUERY_KEY(PIPELINE_ID), [trigger({ enabled: true })]);

    const { result } = renderHook(() => useSetTriggerEnabled(PIPELINE_ID), { wrapper: Wrapper });

    const mutatePromise = result.current.mutateAsync({ triggerId: 'trigger-1', enabled: false });

    await waitFor(() => {
      const cached = queryClient.getQueryData<TriggerEntity[]>(TRIGGERS_QUERY_KEY(PIPELINE_ID));
      expect(cached?.[0].enabled).toBe(false);
    });
    // The repository call is still pending at this point.
    expect(toastSuccess).not.toHaveBeenCalled();

    resolveCall!(ScyllaResult.success(trigger({ id: 'trigger-1', enabled: false })));
    await mutatePromise;
    expect(setEnabled).toHaveBeenCalledWith('trigger-1', false);
    expect(toastSuccess).toHaveBeenCalledWith('Trigger disabled');
  });

  it('picks the enabled vs disabled toast based on the target value', async () => {
    const { repository } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    queryClient.setQueryData(TRIGGERS_QUERY_KEY(PIPELINE_ID), [trigger({ enabled: false })]);
    const { result } = renderHook(() => useSetTriggerEnabled(PIPELINE_ID), { wrapper: Wrapper });

    await result.current.mutateAsync({ triggerId: 'trigger-1', enabled: true });

    expect(toastSuccess).toHaveBeenCalledWith('Trigger enabled');
  });

  it('rolls back the optimistic update if the repository call fails', async () => {
    const error = new ScyllaError('boom', { cause: { code: 'INTERNAL' } });
    const { repository } = makeFakeRepository({
      setEnabled: vi.fn().mockResolvedValue(ScyllaResult.error(error)),
    });
    const { Wrapper, queryClient } = wrapperFor(repository);
    queryClient.setQueryData(TRIGGERS_QUERY_KEY(PIPELINE_ID), [trigger({ enabled: true })]);
    const { result } = renderHook(() => useSetTriggerEnabled(PIPELINE_ID), { wrapper: Wrapper });

    await expect(
      result.current.mutateAsync({ triggerId: 'trigger-1', enabled: false }),
    ).rejects.toBe(error);

    await waitFor(() => {
      const cached = queryClient.getQueryData<TriggerEntity[]>(TRIGGERS_QUERY_KEY(PIPELINE_ID));
      expect(cached?.[0].enabled).toBe(true);
    });
    expect(toastSuccess).not.toHaveBeenCalled();
  });
});
