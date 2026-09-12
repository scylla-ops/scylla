import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { TriggerFormDialog } from './TriggerFormDialog';
import { TriggerKind } from '@/modules/features/triggers/domain/structs/trigger-source.struct.ts';
import type { TriggerEntity } from '@/modules/features/triggers/domain/entities/trigger.entity.ts';

const createMutateAsync = vi.fn();
const updateMutateAsync = vi.fn();
vi.mock('@/modules/features/triggers/presentation/hooks/use-create-trigger.ts', () => ({
  useCreateTrigger: () => ({ mutateAsync: createMutateAsync, isPending: false }),
}));
vi.mock('@/modules/features/triggers/presentation/hooks/use-update-trigger.ts', () => ({
  useUpdateTrigger: () => ({ mutateAsync: updateMutateAsync, isPending: false }),
}));

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

const trigger = (overrides: Partial<TriggerEntity> = {}): TriggerEntity => ({
  id: 't1',
  pipelineId: 'p1',
  name: 'nightly-build',
  source: { kind: TriggerKind.Cron, expression: '0 9 * * *' },
  inputs: [],
  enabled: true,
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

beforeEach(() => {
  createMutateAsync.mockReset();
  updateMutateAsync.mockReset();
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
  Element.prototype.hasPointerCapture = vi.fn().mockReturnValue(false);
  Element.prototype.releasePointerCapture = vi.fn();
  Element.prototype.scrollIntoView = vi.fn();
});

describe('TriggerFormDialog', () => {
  it('create mode: titles the dialog "New trigger" and offers a kind picker', () => {
    renderWithI18n(<TriggerFormDialog open onOpenChange={vi.fn()} pipelineId='p1' />);
    expect(screen.getByText('New trigger')).toBeInTheDocument();
    expect(screen.getByRole('combobox', { name: 'Type' })).toBeInTheDocument();
  });

  it('edit mode: titles the dialog "Edit trigger" and locks the kind behind a badge', () => {
    renderWithI18n(<TriggerFormDialog open onOpenChange={vi.fn()} pipelineId='p1' trigger={trigger()} />);
    expect(screen.getByText('Edit trigger')).toBeInTheDocument();
    expect(screen.getByText('Cron')).toBeInTheDocument();
    // The kind picker Select is gone entirely - CronScheduleBuilder's own
    // hour/minute Selects are still there, so this can't just check "no
    // combobox at all".
    expect(screen.queryByRole('combobox', { name: 'Type' })).not.toBeInTheDocument();
  });

  it('Create stays disabled until a name is entered', async () => {
    const user = userEvent.setup();
    renderWithI18n(<TriggerFormDialog open onOpenChange={vi.fn()} pipelineId='p1' />);
    const button = screen.getByRole('button', { name: 'Create' });
    expect(button).toBeDisabled();

    await user.type(screen.getByLabelText('Name'), 'nightly-build');
    expect(button).toBeEnabled();
  });

  it('an edited trigger with an unknown source can never be saved', () => {
    renderWithI18n(
      <TriggerFormDialog
        open
        onOpenChange={vi.fn()}
        pipelineId='p1'
        trigger={trigger({ source: { kind: TriggerKind.Unknown } })}
      />,
    );
    expect(screen.getByText('Unknown')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
  });

  it('switching kind to Webhook swaps the schedule builder for a signature-header input', async () => {
    const user = userEvent.setup();
    renderWithI18n(<TriggerFormDialog open onOpenChange={vi.fn()} pipelineId='p1' />);

    await user.click(screen.getByRole('combobox', { name: 'Type' }));
    await user.click(await screen.findByText('Webhook'));

    expect(screen.getByLabelText('Signature header')).toBeInTheDocument();
    expect(screen.queryByText('Schedule')).not.toBeInTheDocument();
  });

  it('creating a trigger calls onCreated with the mutation result and closes the dialog', async () => {
    createMutateAsync.mockResolvedValue({ trigger: trigger(), webhookSecret: undefined });
    const onOpenChange = vi.fn();
    const onCreated = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(
      <TriggerFormDialog open onOpenChange={onOpenChange} pipelineId='p1' onCreated={onCreated} />,
    );

    await user.type(screen.getByLabelText('Name'), 'nightly-build');
    await user.click(screen.getByRole('button', { name: 'Create' }));

    expect(createMutateAsync).toHaveBeenCalledWith(
      expect.objectContaining({ name: 'nightly-build', source: expect.objectContaining({ kind: 'cron' }) }),
    );
    expect(onCreated).toHaveBeenCalledWith({ trigger: trigger(), webhookSecret: undefined });
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it('editing a trigger updates it by id and never calls onCreated', async () => {
    updateMutateAsync.mockResolvedValue(undefined);
    const onOpenChange = vi.fn();
    const onCreated = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(
      <TriggerFormDialog
        open
        onOpenChange={onOpenChange}
        pipelineId='p1'
        trigger={trigger({ id: 't-77' })}
        onCreated={onCreated}
      />,
    );

    await user.click(screen.getByRole('button', { name: 'Save' }));

    expect(updateMutateAsync).toHaveBeenCalledWith(
      expect.objectContaining({ triggerId: 't-77' }),
    );
    expect(onCreated).not.toHaveBeenCalled();
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it('Cancel closes the dialog without submitting', async () => {
    const onOpenChange = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<TriggerFormDialog open onOpenChange={onOpenChange} pipelineId='p1' />);

    await user.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(onOpenChange).toHaveBeenCalledWith(false);
    expect(createMutateAsync).not.toHaveBeenCalled();
  });
});
