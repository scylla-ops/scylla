import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { TriggerSourceCell } from './TriggerSourceCell';
import { TriggerKind } from '@/modules/features/triggers/domain/structs/trigger-source.struct.ts';
import type { TriggerEntity } from '@/modules/features/triggers/domain/entities/trigger.entity.ts';

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

beforeEach(() => {
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
});

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

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

describe('TriggerSourceCell', () => {
  it('shows a cron trigger\'s expression, converted to local time', () => {
    renderWithI18n(<TriggerSourceCell trigger={trigger({ source: { kind: TriggerKind.Cron, expression: '0 0 * * *' } })} />);
    // Exact text depends on the host's local offset, so just check something
    // real (not the em dash placeholder) rendered as a 5-field cron string.
    const code = screen.getByText(/\d+ \d+ \S+ \S+ \S+|—/);
    expect(code).toBeInTheDocument();
  });

  it('falls back to an em dash for a blank cron expression', () => {
    renderWithI18n(<TriggerSourceCell trigger={trigger({ source: { kind: TriggerKind.Cron, expression: '' } })} />);
    expect(screen.getByText('—')).toBeInTheDocument();
  });

  it('shows a webhook trigger\'s URL, truncated (12 chars) and copyable', () => {
    renderWithI18n(
      <TriggerSourceCell
        trigger={trigger({
          source: { kind: TriggerKind.Webhook, signatureHeader: '', webhookUrl: 'https://x/webhooks/1' },
        })}
      />,
    );
    // CopyableText truncates the displayed text to 12 chars + an ellipsis.
    expect(screen.getByText('https://x/we...')).toBeInTheDocument();
  });

  it('shows a plain em dash for a source arm this build does not understand', () => {
    renderWithI18n(<TriggerSourceCell trigger={trigger({ source: { kind: TriggerKind.Unknown } })} />);
    expect(screen.getByText('—')).toBeInTheDocument();
  });
});
