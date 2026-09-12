import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { TriggersOverview } from './TriggersOverview';
import { TriggerKind } from '@/modules/features/triggers/domain/structs/trigger-source.struct.ts';
import type { TriggerEntity } from '@/modules/features/triggers/domain/entities/trigger.entity.ts';

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

const trigger = (overrides: Partial<TriggerEntity> = {}): TriggerEntity => ({
  id: 't1',
  pipelineId: 'p1',
  name: 'nightly',
  source: { kind: TriggerKind.Cron, expression: '0 9 * * *' },
  inputs: [],
  enabled: true,
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

describe('TriggersOverview', () => {
  it('shows how many of the triggers are enabled', () => {
    renderWithI18n(
      <TriggersOverview
        triggers={[trigger({ enabled: true }), trigger({ id: 't2', enabled: false })]}
      />,
    );
    expect(screen.getByText('1/2')).toBeInTheDocument();
  });

  it('counts only webhook-sourced triggers as webhook endpoints', () => {
    renderWithI18n(
      <TriggersOverview
        triggers={[
          trigger({ source: { kind: TriggerKind.Webhook, signatureHeader: '', webhookUrl: 'x' } }),
          trigger({ id: 't2', source: { kind: TriggerKind.Cron, expression: '0 9 * * *' } }),
        ]}
      />,
    );
    expect(screen.getByText('1')).toBeInTheDocument();
  });

  it('shows a dash for the next scheduled run when there is none', () => {
    renderWithI18n(<TriggersOverview triggers={[trigger({ enabled: false })]} />);
    expect(screen.getByText('—')).toBeInTheDocument();
  });

  it('ignores a disabled cron trigger and a webhook trigger when picking the next run', () => {
    renderWithI18n(
      <TriggersOverview
        triggers={[
          trigger({ enabled: false, nextFireAt: '2026-01-01T00:00:00.000Z' }),
          trigger({
            id: 't2',
            source: { kind: TriggerKind.Webhook, signatureHeader: '', webhookUrl: 'x' },
            nextFireAt: '2026-01-02T00:00:00.000Z',
          }),
        ]}
      />,
    );
    expect(screen.getByText('—')).toBeInTheDocument();
  });

  it('picks the earliest next-fire time among enabled cron triggers', () => {
    renderWithI18n(
      <TriggersOverview
        triggers={[
          trigger({ id: 't1', nextFireAt: '2026-03-01T00:00:00.000Z' }),
          trigger({ id: 't2', nextFireAt: '2026-01-15T00:00:00.000Z' }),
        ]}
      />,
    );
    // formatDate's exact wording is covered elsewhere - just that the earlier
    // (January) date, not the March one, is the one shown.
    expect(screen.queryByText('—')).not.toBeInTheDocument();
    expect(screen.getByText(/jan/i)).toBeInTheDocument();
  });
});
