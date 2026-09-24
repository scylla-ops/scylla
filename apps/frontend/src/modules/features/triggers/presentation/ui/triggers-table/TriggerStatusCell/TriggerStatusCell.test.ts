import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/svelte';
import { render } from '@/test/render.svelte.ts';
import { TriggerKind } from '../../../../domain/structs/trigger-source.struct.ts';
import type { TriggerEntity } from '../../../../domain/entities/trigger.entity.ts';
import TriggerStatusCell from './TriggerStatusCell.svelte';

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

const webhook = { kind: TriggerKind.Webhook, signatureHeader: '', webhookUrl: 'https://x/y' } as const;

describe('TriggerStatusCell', () => {
  it('shows "Never fired" when there is no lastResult and no lastFiredAt', () => {
    render(TriggerStatusCell, { trigger: trigger() });
    expect(screen.getByText('Never fired')).toBeInTheDocument();
  });

  it('shows "Error" for a failed last fire', () => {
    render(TriggerStatusCell, {
      trigger: trigger({
        lastFiredAt: '2026-01-01T00:00:00.000Z',
        lastResult: { kind: 'failed', error: 'timeout' },
      }),
    });
    expect(screen.getByText('Error')).toBeInTheDocument();
  });

  it('carries the failure reason as the badge title, so it is not lost', () => {
    render(TriggerStatusCell, {
      trigger: trigger({
        lastFiredAt: '2026-01-01T00:00:00.000Z',
        lastResult: { kind: 'failed', error: 'timeout' },
      }),
    });
    expect(screen.getByTitle('timeout')).toBeInTheDocument();
  });

  it('shows "OK" for a succeeded last fire', () => {
    render(TriggerStatusCell, {
      trigger: trigger({
        lastFiredAt: '2026-01-01T00:00:00.000Z',
        lastResult: { kind: 'succeeded' },
      }),
    });
    expect(screen.getByText('OK')).toBeInTheDocument();
  });

  it('shows "Unknown" for a fired trigger whose outcome arm this build does not recognize', () => {
    render(TriggerStatusCell, {
      trigger: trigger({
        lastFiredAt: '2026-01-01T00:00:00.000Z',
        lastResult: { kind: 'unknown' },
      }),
    });
    expect(screen.getByText('Unknown')).toBeInTheDocument();
  });

  it('has fired (not "Never fired") when only lastFiredAt is set, with no lastResult', () => {
    render(TriggerStatusCell, { trigger: trigger({ lastFiredAt: '2026-01-01T00:00:00.000Z' }) });

    expect(screen.queryByText('Never fired')).not.toBeInTheDocument();
    expect(screen.getByText('Unknown')).toBeInTheDocument();
  });

  it('a disabled webhook trigger explains the URL returns 404', () => {
    render(TriggerStatusCell, { trigger: trigger({ enabled: false, source: webhook }) });
    expect(screen.getByText('disabled — URL returns 404')).toBeInTheDocument();
  });

  it('a disabled cron trigger just says "disabled"', () => {
    render(TriggerStatusCell, { trigger: trigger({ enabled: false }) });
    expect(screen.getByText('disabled')).toBeInTheDocument();
  });

  it('an enabled cron trigger with a next fire time shows "next <date>"', () => {
    render(TriggerStatusCell, { trigger: trigger({ nextFireAt: '2026-06-01T00:00:00.000Z' }) });
    expect(screen.getByText('next', { exact: false })).toBeInTheDocument();
  });

  it('an enabled cron trigger with no next fire time shows nothing extra', () => {
    render(TriggerStatusCell, { trigger: trigger({ nextFireAt: undefined }) });
    expect(screen.queryByText('next', { exact: false })).not.toBeInTheDocument();
  });

  it('an enabled webhook trigger that has fired shows "last <date>"', () => {
    render(TriggerStatusCell, {
      trigger: trigger({
        source: webhook,
        lastFiredAt: '2026-06-01T00:00:00.000Z',
        lastResult: { kind: 'succeeded' },
      }),
    });
    expect(screen.getByText('last', { exact: false })).toBeInTheDocument();
  });
});
