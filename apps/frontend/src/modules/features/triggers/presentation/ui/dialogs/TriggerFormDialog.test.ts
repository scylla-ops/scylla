import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { findFloating, focusSettled, render, withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { TriggersRepository } from '../../../domain/repository/triggers.repository.ts';
import type { TriggerEntity } from '../../../domain/entities/trigger.entity.ts';
import { TriggerKind } from '../../../domain/structs/trigger-source.struct.ts';
import TriggerFormDialog from './TriggerFormDialog.svelte';

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

let create: ReturnType<typeof vi.fn>;
let update: ReturnType<typeof vi.fn>;
let cache: ReturnType<typeof withQueryClient>;
let restoreRegistry: () => void;

beforeEach(() => {
  create = vi.fn().mockResolvedValue(ScyllaResult.success({ trigger: trigger() }));
  update = vi.fn().mockResolvedValue(ScyllaResult.success(trigger()));

  cache = withQueryClient();
  restoreRegistry = withRegistry({
    triggers: { triggersRepository: { create, update } as unknown as TriggersRepository },
  });
});

afterEach(() => {
  cache.restore();
  restoreRegistry();
});

const props = (overrides: Record<string, unknown> = {}) => ({
  open: true,
  onOpenChange: vi.fn(),
  pipelineId: 'pipeline-1',
  ...overrides,
});

const submitButton = (name: 'Create' | 'Save') => screen.getByRole('button', { name });

describe('TriggerFormDialog — create mode', () => {
  it('offers a blank create form with the kind still choosable', () => {
    render(TriggerFormDialog, props());

    expect(screen.getByText('New trigger')).toBeInTheDocument();
    expect(screen.getByLabelText('Name')).toHaveValue('');
    expect(screen.getByLabelText('Type')).toBeInTheDocument();
  });

  it('refuses to submit without a name', () => {
    render(TriggerFormDialog, props());
    expect(submitButton('Create')).toBeDisabled();
  });

  it('creates a cron trigger with the name and schedule that were entered', async () => {
    render(TriggerFormDialog, props());
    await focusSettled();

    await userEvent.type(screen.getByLabelText('Name'), 'nightly-build');
    expect(screen.getByLabelText('Name')).toHaveValue('nightly-build');
    await vi.waitFor(() => expect(submitButton('Create')).toBeEnabled());
    await userEvent.click(submitButton('Create'));

    await vi.waitFor(() => expect(create).toHaveBeenCalled());
    const [pipelineId, draft] = create.mock.calls[0];
    expect(pipelineId).toBe('pipeline-1');
    expect(draft.name).toBe('nightly-build');
    expect(draft.source.kind).toBe(TriggerKind.Cron);
  });

  it('hands the created trigger back so a one-time webhook secret can be revealed', async () => {
    const created = { trigger: trigger(), webhookSecret: 'whsec-once' };
    create.mockResolvedValue(ScyllaResult.success(created));
    const onCreated = vi.fn();
    render(TriggerFormDialog, props({ onCreated }));
    await focusSettled();

    await userEvent.type(screen.getByLabelText('Name'), 'hook');
    await userEvent.click(submitButton('Create'));

    await vi.waitFor(() => expect(onCreated).toHaveBeenCalledWith(created));
  });

  it('closes once the trigger is in', async () => {
    const onOpenChange = vi.fn();
    render(TriggerFormDialog, props({ onOpenChange }));
    await focusSettled();

    await userEvent.type(screen.getByLabelText('Name'), 'hook');
    await userEvent.click(submitButton('Create'));

    await vi.waitFor(() => expect(onOpenChange).toHaveBeenCalledWith(false));
  });

  it('stays open when the create was refused, so the entry is not lost', async () => {
    create.mockRejectedValue(new Error('denied'));
    const onOpenChange = vi.fn();
    render(TriggerFormDialog, props({ onOpenChange }));
    await focusSettled();

    await userEvent.type(screen.getByLabelText('Name'), 'hook');
    await userEvent.click(submitButton('Create'));

    await vi.waitFor(() => expect(create).toHaveBeenCalled());
    expect(onOpenChange).not.toHaveBeenCalledWith(false);
  });

  it('swaps the schedule builder for a signature header when the kind is webhook', async () => {
    render(TriggerFormDialog, props());
    await focusSettled();

    await userEvent.click(screen.getByLabelText('Type'));
    await userEvent.click(await findFloating('option', 'Webhook'));

    expect(screen.getByLabelText('Signature header')).toBeInTheDocument();
    expect(screen.queryByRole('radio', { name: /daily/i })).not.toBeInTheDocument();
  });

  it('cancel closes without writing anything', async () => {
    const onOpenChange = vi.fn();
    render(TriggerFormDialog, props({ onOpenChange }));

    await userEvent.click(screen.getByRole('button', { name: 'Cancel' }));

    expect(onOpenChange).toHaveBeenCalledWith(false);
    expect(create).not.toHaveBeenCalled();
  });
});

describe('TriggerFormDialog — edit mode', () => {
  it('seeds the form from the trigger and locks its kind', () => {
    render(TriggerFormDialog, props({ trigger: trigger({ name: 'nightly' }) }));

    expect(screen.getByText('Edit trigger')).toBeInTheDocument();
    expect(screen.getByLabelText('Name')).toHaveValue('nightly');
    expect(screen.getByText("Type can't be changed — delete and recreate to switch.")).toBeInTheDocument();
  });

  it('updates by trigger id rather than creating a second one', async () => {
    render(TriggerFormDialog, props({ trigger: trigger({ id: 'trigger-9' }) }));
    await focusSettled();

    await userEvent.type(screen.getByLabelText('Name'), '-v2');
    await userEvent.click(submitButton('Save'));

    await vi.waitFor(() => expect(update).toHaveBeenCalled());
    const [triggerId, draft] = update.mock.calls[0];
    expect(triggerId).toBe('trigger-9');
    expect(draft.name).toBe('nightly-v2');
    expect(create).not.toHaveBeenCalled();
  });

  it('refuses to re-send a source arm this build cannot render', () => {
    render(TriggerFormDialog, props({ trigger: trigger({ source: { kind: TriggerKind.Unknown } }) }));

    // Round-tripping a shape we do not understand would silently rewrite it.
    expect(submitButton('Save')).toBeDisabled();
    expect(screen.getByText('Unknown')).toBeInTheDocument();
  });

  it('keeps a webhook trigger on its signature header field', () => {
    render(
      TriggerFormDialog,
      props({
        trigger: trigger({
          source: {
            kind: TriggerKind.Webhook,
            signatureHeader: 'X-Hub-Signature-256',
            webhookUrl: 'https://x/y',
          },
        }),
      }),
    );

    expect(screen.getByLabelText('Signature header')).toHaveValue('X-Hub-Signature-256');
  });
});
