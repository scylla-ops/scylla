// @vitest-environment node
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { QueryClient } from '@tanstack/query-core';
import { setDependencyRegistry } from '@platform/di';
import { setQueryClient } from '@platform/query';
import { runMutationFn, runQueryFn } from '@/test/queries.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { TriggersRepository } from '../domain/repository/triggers.repository.ts';
import type { TriggerDraft, TriggerEntity } from '../domain/entities/trigger.entity.ts';
import { TriggerKind } from '../domain/structs/trigger-source.struct.ts';
import { TRIGGERS_QUERY_KEY, triggerMutations, triggerQueries } from './triggers.queries.ts';

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

const cronTrigger = (overrides: Partial<TriggerEntity> = {}): TriggerEntity =>
  trigger({ source: { kind: TriggerKind.Cron, expression: '0 * * * *' }, ...overrides });

const draft: TriggerDraft = {
  name: 'github-push',
  source: { kind: TriggerKind.Webhook, signatureHeader: '' },
  inputs: [],
};

/** Stands in for the query TanStack hands `refetchInterval`. */
const queryWithData = (data: TriggerEntity[]) => ({ state: { data } });

const pollFor = (triggers: TriggerEntity[]) => {
  const { refetchInterval } = triggerQueries.byPipeline(PIPELINE_ID);
  if (typeof refetchInterval !== 'function') {
    throw new Error('byPipeline should decide its interval from the cached data.');
  }
  return refetchInterval(queryWithData(triggers) as never);
};

let repository: TriggersRepository;
let queryClient: QueryClient;

beforeEach(() => {
  repository = {
    listByPipelineId: vi.fn().mockResolvedValue(ScyllaResult.success([trigger()])),
    getById: vi.fn().mockResolvedValue(ScyllaResult.success(trigger())),
    create: vi.fn().mockResolvedValue(ScyllaResult.success({ trigger: trigger() })),
    update: vi.fn().mockResolvedValue(ScyllaResult.success(trigger())),
    deleteById: vi.fn().mockResolvedValue(ScyllaResult.success(undefined)),
    setEnabled: vi
      .fn()
      .mockImplementation((id: string, enabled: boolean) =>
        Promise.resolve(ScyllaResult.success(trigger({ id, enabled }))),
      ),
    fireNow: vi.fn().mockResolvedValue(ScyllaResult.success('job-1')),
  };

  setDependencyRegistry({ triggers: { triggersRepository: repository } });

  queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  setQueryClient(queryClient);
});

afterEach(() => {
  setDependencyRegistry(null);
  setQueryClient(null);
});

describe('triggerQueries.byPipeline', () => {
  it("lists a pipeline's triggers under the pipeline-scoped key", async () => {
    const options = triggerQueries.byPipeline(PIPELINE_ID);

    expect(options.queryKey).toEqual(TRIGGERS_QUERY_KEY(PIPELINE_ID));
    await expect(runQueryFn(options)).resolves.toEqual([trigger()]);
    expect(repository.listByPipelineId).toHaveBeenCalledWith(PIPELINE_ID);
  });

  it('stays disabled without a pipeline id, rather than fetching for an empty one', () => {
    expect(triggerQueries.byPipeline('').enabled).toBe(false);
    expect(triggerQueries.byPipeline(PIPELINE_ID).enabled).toBe(true);
  });

  it('gives two pipelines two distinct cache entries', () => {
    expect(TRIGGERS_QUERY_KEY('pipeline-1')).not.toEqual(TRIGGERS_QUERY_KEY('pipeline-2'));
  });

  it('polls while an enabled cron trigger exists — its next fire moves on its own', () => {
    expect(pollFor([cronTrigger({ enabled: true, nextFireAt: '2026-01-02T00:00:00.000Z' })])).toBe(
      30_000,
    );
  });

  it('stops polling for a disabled cron trigger', () => {
    expect(pollFor([cronTrigger({ enabled: false })])).toBe(false);
  });

  it('never polls for webhooks alone — no interval predicts an inbound call', () => {
    expect(pollFor([trigger({ enabled: true })])).toBe(false);
  });

  it('polls when a cron trigger sits alongside webhooks', () => {
    expect(pollFor([trigger({ enabled: true }), cronTrigger({ enabled: true })])).toBe(30_000);
  });

  it('does not poll an empty list', () => {
    expect(pollFor([])).toBe(false);
  });
});

describe('triggerMutations', () => {
  it('creates a trigger on the pipeline and answers the one-time secret', async () => {
    repository.create = vi
      .fn()
      .mockResolvedValue(ScyllaResult.success({ trigger: trigger(), webhookSecret: 'whsec-once' }));

    const options = triggerMutations.create(PIPELINE_ID);
    const created = await runMutationFn(options, draft);

    expect(repository.create).toHaveBeenCalledWith(PIPELINE_ID, draft);
    expect(created.webhookSecret).toBe('whsec-once');
  });

  it('invalidates the list after a create', () => {
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries');
    const options = triggerMutations.create(PIPELINE_ID);

    options.onSuccess?.({ trigger: trigger() }, draft, undefined, undefined as never);

    expect(invalidate).toHaveBeenCalledWith({ queryKey: TRIGGERS_QUERY_KEY(PIPELINE_ID) });
  });

  it('updates by trigger id, not by pipeline id', async () => {
    const options = triggerMutations.update(PIPELINE_ID);

    await runMutationFn(options, { triggerId: 'trigger-1', draft });

    expect(repository.update).toHaveBeenCalledWith('trigger-1', draft);
  });

  it('deletes by trigger id and refreshes the list', async () => {
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries');
    const options = triggerMutations.remove(PIPELINE_ID);

    await runMutationFn(options, 'trigger-1');
    expect(repository.deleteById).toHaveBeenCalledWith('trigger-1');

    options.onSuccess?.(undefined, 'trigger-1', undefined, undefined as never);
    expect(invalidate).toHaveBeenCalledWith({ queryKey: TRIGGERS_QUERY_KEY(PIPELINE_ID) });
  });

  it('fires a trigger and invalidates the jobs list too — the run must appear', async () => {
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries');
    const options = triggerMutations.fireNow(PIPELINE_ID);

    const jobId = await runMutationFn(options, 'trigger-1');
    expect(repository.fireNow).toHaveBeenCalledWith('trigger-1');
    expect(jobId).toBe('job-1');

    options.onSuccess?.('job-1', 'trigger-1', undefined, undefined as never);
    expect(invalidate).toHaveBeenCalledWith({ queryKey: ['jobs', 'pipeline', PIPELINE_ID] });
    expect(invalidate).toHaveBeenCalledWith({ queryKey: TRIGGERS_QUERY_KEY(PIPELINE_ID) });
  });
});

describe('triggerMutations.setEnabled', () => {
  const seedCache = (enabled: boolean) =>
    queryClient.setQueryData(TRIGGERS_QUERY_KEY(PIPELINE_ID), [trigger({ enabled })]);

  const cached = () =>
    queryClient.getQueryData<TriggerEntity[]>(TRIGGERS_QUERY_KEY(PIPELINE_ID)) ?? [];

  it('flips the cached switch before the call resolves', async () => {
    seedCache(true);
    const options = triggerMutations.setEnabled(PIPELINE_ID);

    // onMutate alone — the repository is never called here, which is the point:
    // the switch has to move under the pointer, not a round trip later.
    await options.onMutate?.({ triggerId: 'trigger-1', enabled: false }, undefined as never);

    expect(cached()[0].enabled).toBe(false);
    expect(repository.setEnabled).not.toHaveBeenCalled();
  });

  it('leaves the other triggers of the pipeline untouched', async () => {
    queryClient.setQueryData(TRIGGERS_QUERY_KEY(PIPELINE_ID), [
      trigger({ id: 'trigger-1', enabled: true }),
      trigger({ id: 'trigger-2', enabled: true }),
    ]);

    const options = triggerMutations.setEnabled(PIPELINE_ID);
    await options.onMutate?.({ triggerId: 'trigger-1', enabled: false }, undefined as never);

    expect(cached().map(entry => entry.enabled)).toEqual([false, true]);
  });

  it('rolls the optimistic toggle back when the call fails', async () => {
    seedCache(true);
    const options = triggerMutations.setEnabled(PIPELINE_ID);

    const context = await options.onMutate?.({ triggerId: 'trigger-1', enabled: false }, undefined as never);
    expect(cached()[0].enabled).toBe(false);

    options.onError?.(
      new Error('boom'),
      { triggerId: 'trigger-1', enabled: false },
      context,
      undefined as never,
    );

    expect(cached()[0].enabled).toBe(true);
  });

  it('sends the target value through to the repository', async () => {
    const options = triggerMutations.setEnabled(PIPELINE_ID);

    await runMutationFn(options, { triggerId: 'trigger-1', enabled: true });

    expect(repository.setEnabled).toHaveBeenCalledWith('trigger-1', true);
  });

  it('refreshes the list once settled, whatever the outcome', () => {
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries');
    const options = triggerMutations.setEnabled(PIPELINE_ID);

    options.onSettled?.(
      undefined,
      new Error('boom'),
      { triggerId: 'trigger-1', enabled: false },
      undefined,
      undefined as never,
    );

    expect(invalidate).toHaveBeenCalledWith({ queryKey: TRIGGERS_QUERY_KEY(PIPELINE_ID) });
  });
});
