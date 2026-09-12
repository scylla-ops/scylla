import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { TriggerStatusCell } from './TriggerStatusCell';
import { TriggerKind } from '@/modules/features/triggers/domain/structs/trigger-source.struct.ts';
import type { TriggerEntity } from '@/modules/features/triggers/domain/entities/trigger.entity.ts';

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

describe('TriggerStatusCell', () => {
  it('shows "Never fired" when there is no lastResult and no lastFiredAt', () => {
    renderWithI18n(<TriggerStatusCell trigger={trigger()} />);
    expect(screen.getByText('Never fired')).toBeInTheDocument();
  });

  it('shows "Error" for a failed last fire', () => {
    renderWithI18n(
      <TriggerStatusCell
        trigger={trigger({ lastFiredAt: '2026-01-01T00:00:00.000Z', lastResult: { kind: 'failed', error: 'timeout' } })}
      />,
    );
    expect(screen.getByText('Error')).toBeInTheDocument();
  });

  it('shows "OK" for a succeeded last fire', () => {
    renderWithI18n(
      <TriggerStatusCell
        trigger={trigger({ lastFiredAt: '2026-01-01T00:00:00.000Z', lastResult: { kind: 'succeeded' } })}
      />,
    );
    expect(screen.getByText('OK')).toBeInTheDocument();
  });

  it('shows "Unknown" for a fired trigger whose outcome arm this build does not recognize', () => {
    renderWithI18n(
      <TriggerStatusCell
        trigger={trigger({ lastFiredAt: '2026-01-01T00:00:00.000Z', lastResult: { kind: 'unknown' } })}
      />,
    );
    expect(screen.getByText('Unknown')).toBeInTheDocument();
  });

  it('has fired (not "Never fired") when only lastFiredAt is set, with no lastResult', () => {
    renderWithI18n(<TriggerStatusCell trigger={trigger({ lastFiredAt: '2026-01-01T00:00:00.000Z' })} />);
    expect(screen.queryByText('Never fired')).not.toBeInTheDocument();
    expect(screen.getByText('Unknown')).toBeInTheDocument();
  });

  it('a disabled webhook trigger explains the URL returns 404', () => {
    renderWithI18n(
      <TriggerStatusCell
        trigger={trigger({
          enabled: false,
          source: { kind: TriggerKind.Webhook, signatureHeader: '', webhookUrl: 'https://x/y' },
        })}
      />,
    );
    expect(screen.getByText('disabled — URL returns 404')).toBeInTheDocument();
  });

  it('a disabled cron trigger just says "disabled"', () => {
    renderWithI18n(<TriggerStatusCell trigger={trigger({ enabled: false })} />);
    expect(screen.getByText('disabled')).toBeInTheDocument();
  });

  it('an enabled cron trigger with a next fire time shows "next <date>"', () => {
    renderWithI18n(<TriggerStatusCell trigger={trigger({ nextFireAt: '2026-06-01T00:00:00.000Z' })} />);
    expect(screen.getByText('next', { exact: false })).toBeInTheDocument();
  });

  it('an enabled cron trigger with no next fire time shows nothing extra', () => {
    renderWithI18n(<TriggerStatusCell trigger={trigger({ nextFireAt: undefined })} />);
    expect(screen.queryByText('next', { exact: false })).not.toBeInTheDocument();
  });

  it('an enabled webhook trigger that has fired shows "last <date>"', () => {
    renderWithI18n(
      <TriggerStatusCell
        trigger={trigger({
          source: { kind: TriggerKind.Webhook, signatureHeader: '', webhookUrl: 'https://x/y' },
          lastFiredAt: '2026-06-01T00:00:00.000Z',
          lastResult: { kind: 'succeeded' },
        })}
      />,
    );
    expect(screen.getByText('last', { exact: false })).toBeInTheDocument();
  });
});
