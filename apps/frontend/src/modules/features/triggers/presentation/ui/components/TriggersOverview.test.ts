import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/svelte';
import { render } from '@/test/render.svelte.ts';
import { TriggerKind } from '../../../domain/structs/trigger-source.struct.ts';
import type { TriggerEntity } from '../../../domain/entities/trigger.entity.ts';
import TriggersOverview from './TriggersOverview.svelte';

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

const webhookSource = {
  kind: TriggerKind.Webhook,
  signatureHeader: '',
  webhookUrl: 'x',
} as const;

describe('TriggersOverview', () => {
  it('shows how many of the triggers are enabled', () => {
    render(TriggersOverview, {
      triggers: [trigger({ enabled: true }), trigger({ id: 't2', enabled: false })],
    });
    expect(screen.getByText('1/2')).toBeInTheDocument();
  });

  it('counts only webhook-sourced triggers as webhook endpoints', () => {
    render(TriggersOverview, {
      triggers: [
        trigger({ source: webhookSource }),
        trigger({ id: 't2', source: { kind: TriggerKind.Cron, expression: '0 9 * * *' } }),
      ],
    });
    expect(screen.getByText('1')).toBeInTheDocument();
  });

  it('shows a dash for the next scheduled run when there is none', () => {
    render(TriggersOverview, { triggers: [trigger({ enabled: false })] });
    expect(screen.getByText('—')).toBeInTheDocument();
  });

  it('ignores a disabled cron trigger and a webhook trigger when picking the next run', () => {
    render(TriggersOverview, {
      triggers: [
        trigger({ enabled: false, nextFireAt: '2026-01-01T00:00:00.000Z' }),
        trigger({ id: 't2', source: webhookSource, nextFireAt: '2026-01-02T00:00:00.000Z' }),
      ],
    });
    expect(screen.getByText('—')).toBeInTheDocument();
  });

  it('picks the earliest next-fire time among enabled cron triggers', () => {
    render(TriggersOverview, {
      triggers: [
        trigger({ id: 't1', nextFireAt: '2026-03-01T00:00:00.000Z' }),
        trigger({ id: 't2', nextFireAt: '2026-01-15T00:00:00.000Z' }),
      ],
    });

    // formatDate's exact wording is covered elsewhere — this only pins that the
    // earlier (January) date won, not the March one.
    expect(screen.queryByText('—')).not.toBeInTheDocument();
    expect(screen.getByText(/jan/i)).toBeInTheDocument();
  });

  it('reads 0/0 for an empty pipeline rather than rendering nothing', () => {
    render(TriggersOverview, { triggers: [] });
    expect(screen.getByText('0/0')).toBeInTheDocument();
  });
});
