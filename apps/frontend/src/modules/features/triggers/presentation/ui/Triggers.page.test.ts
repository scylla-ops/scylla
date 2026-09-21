import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { PermissionScope, usePermissionsStore } from '@platform/authz';
import { useContextStore } from '@platform/context';
import { useSelectionStore } from '@shared/presentation/stores/use-selection.store.ts';
import { focusSettled, render, withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { ScyllaError, ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { TriggersRepository } from '../../domain/repository/triggers.repository.ts';
import type { TriggerEntity } from '../../domain/entities/trigger.entity.ts';
import { TriggerKind } from '../../domain/structs/trigger-source.struct.ts';
import TriggersPage from './Triggers.page.svelte';

const trigger = (overrides: Partial<TriggerEntity> = {}): TriggerEntity => ({
  id: 'trigger-1',
  pipelineId: 'pipeline-1',
  name: 'nightly',
  source: { kind: TriggerKind.Cron, expression: '0 9 * * *' },
  inputs: [],
  enabled: true,
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

let listByPipelineId: ReturnType<typeof vi.fn>;
let create: ReturnType<typeof vi.fn>;
let cache: ReturnType<typeof withQueryClient>;
let restoreRegistry: () => void;

const props = { pipelineId: 'pipeline-1', projectId: 'project-1' };

beforeEach(() => {
  listByPipelineId = vi.fn().mockResolvedValue(ScyllaResult.success([trigger()]));
  create = vi.fn().mockResolvedValue(ScyllaResult.success({ trigger: trigger() }));

  cache = withQueryClient();
  restoreRegistry = withRegistry({
    triggers: {
      triggersRepository: {
        listByPipelineId,
        create,
        update: vi.fn(),
        deleteById: vi.fn(),
        setEnabled: vi.fn(),
        fireNow: vi.fn(),
        getById: vi.fn(),
      } as unknown as TriggersRepository,
    },
  });

  useContextStore.setState({
    organization: { id: 'org-1', name: 'Acme' },
    project: { id: 'project-1', name: 'Web' },
    pipeline: { id: 'pipeline-1', name: 'build' },
  } as never);
  useSelectionStore.setState({ selectedIds: {} });
  usePermissionsStore.setState({
    permissions: {
      scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }],
    },
  });
});

afterEach(() => {
  cache.restore();
  restoreRegistry();
  usePermissionsStore.setState({ permissions: null });
});

describe('TriggersPage', () => {
  it("lists the pipeline's triggers", async () => {
    render(TriggersPage, props);

    expect(await screen.findByText('nightly')).toBeInTheDocument();
    expect(listByPipelineId).toHaveBeenCalledWith('pipeline-1');
  });

  it('summarises the triggers above the table', async () => {
    render(TriggersPage, props);

    expect(await screen.findByText('1/1')).toBeInTheDocument();
  });

  it('explains an empty pipeline instead of showing a bare table', async () => {
    listByPipelineId.mockResolvedValue(ScyllaResult.success([]));
    render(TriggersPage, props);

    expect(await screen.findByText('No triggers yet')).toBeInTheDocument();
    // No overview either: there is nothing to summarise.
    expect(screen.queryByText('0/0')).not.toBeInTheDocument();
  });

  it('refuses to render without the route parameters it needs', () => {
    render(TriggersPage, {});

    expect(screen.getByText('Pipeline ID is missing')).toBeInTheDocument();
    expect(listByPipelineId).not.toHaveBeenCalled();
  });

  it("surfaces the backend's own message when the list fails", async () => {
    const error = new ScyllaError('nope', { cause: { code: 'PERMISSION_DENIED' } });
    listByPipelineId.mockResolvedValue(ScyllaResult.error(error));
    render(TriggersPage, props);

    expect(await screen.findByText(error.userMessage())).toBeInTheDocument();
  });

  it('reveals a webhook secret exactly once, after creating a webhook trigger', async () => {
    const created = trigger({ id: 'trigger-9', name: 'gh-push' });
    create.mockResolvedValue(
      ScyllaResult.success({ trigger: created, webhookSecret: 'whsec-once' }),
    );
    render(TriggersPage, props);
    await screen.findByText('nightly');

    await userEvent.click(screen.getByRole('button', { name: 'New trigger' }));
    await focusSettled();
    await userEvent.type(await screen.findByLabelText('Name'), 'gh-push');
    await userEvent.click(screen.getByRole('button', { name: 'Create' }));

    // Returned once by the backend — the dialog is the only place it is shown.
    expect(await screen.findByText('whsec-once')).toBeInTheDocument();
  });

  it('reveals nothing when the created trigger carries no secret', async () => {
    create.mockResolvedValue(ScyllaResult.success({ trigger: trigger({ id: 'trigger-9' }) }));
    render(TriggersPage, props);
    await screen.findByText('nightly');

    await userEvent.click(screen.getByRole('button', { name: 'New trigger' }));
    await focusSettled();
    await userEvent.type(await screen.findByLabelText('Name'), 'cron-job');
    await userEvent.click(screen.getByRole('button', { name: 'Create' }));

    await vi.waitFor(() => expect(create).toHaveBeenCalled());
    expect(screen.queryByText('whsec-once')).not.toBeInTheDocument();
  });
});
