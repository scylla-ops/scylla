import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/svelte';
import { render } from '@/test/render.svelte.ts';
import { TriggerKind } from '../../../../domain/structs/trigger-source.struct.ts';
import type { TriggerEntity } from '../../../../domain/entities/trigger.entity.ts';
import TriggerSourceCell from './TriggerSourceCell.svelte';

const trigger = (overrides: Partial<TriggerEntity> = {}): TriggerEntity => ({
  id: 'trigger-1',
  pipelineId: 'pipeline-1',
  name: 'nightly',
  source: { kind: TriggerKind.Cron, expression: '0 0 * * *' },
  inputs: [],
  enabled: true,
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

describe('TriggerSourceCell', () => {
  it("shows a cron trigger's expression, converted to local time", () => {
    render(TriggerSourceCell, {
      trigger: trigger({ source: { kind: TriggerKind.Cron, expression: '0 0 * * *' } }),
    });

    // The exact text depends on the host's local offset, so this only pins
    // that a real 5-field expression rendered rather than the em-dash fallback.
    expect(screen.getByText(/^\S+ \S+ \S+ \S+ \S+$/)).toBeInTheDocument();
    expect(screen.queryByText('—')).not.toBeInTheDocument();
  });

  it('falls back to an em dash for a blank cron expression', () => {
    render(TriggerSourceCell, {
      trigger: trigger({ source: { kind: TriggerKind.Cron, expression: '' } }),
    });
    expect(screen.getByText('—')).toBeInTheDocument();
  });

  it("shows a webhook trigger's full URL", () => {
    render(TriggerSourceCell, {
      trigger: trigger({
        source: {
          kind: TriggerKind.Webhook,
          signatureHeader: '',
          webhookUrl: 'https://x/webhooks/1',
        },
      }),
    });

    // No fixed truncate(): the column's width decides via CSS ellipsis, so the
    // full string is what is actually in the DOM.
    expect(screen.getByText('https://x/webhooks/1')).toBeInTheDocument();
  });

  it('offers a named control to copy a webhook URL', () => {
    render(TriggerSourceCell, {
      trigger: trigger({
        source: {
          kind: TriggerKind.Webhook,
          signatureHeader: '',
          webhookUrl: 'https://x/webhooks/1',
        },
      }),
    });

    expect(screen.getByRole('button', { name: 'Copy url' })).toBeInTheDocument();
  });

  it('shows a plain em dash for a source arm this build does not understand', () => {
    render(TriggerSourceCell, { trigger: trigger({ source: { kind: TriggerKind.Unknown } }) });

    expect(screen.getByText('—')).toBeInTheDocument();
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });
});
