import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { PipelineLastJob } from './PipelineLastJob';
import type { JobEntity } from '@/modules/features/jobs';

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

const job = (overrides: Partial<JobEntity> = {}): JobEntity => ({
  id: 'job-1',
  pipelineId: 'p1',
  status: 'completed',
  nodeExecutions: [],
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:10.000Z',
  ...overrides,
});

describe('PipelineLastJob', () => {
  it('says "No jobs yet" with an empty list', () => {
    renderWithI18n(<PipelineLastJob jobs={[]} />);
    expect(screen.getByText('No jobs yet')).toBeInTheDocument();
    expect(screen.getByText('-')).toBeInTheDocument();
  });

  it('uses the first (most recent) job for its duration and relative time', () => {
    renderWithI18n(
      <PipelineLastJob
        jobs={[
          job({ startedAt: '2026-01-01T00:00:00.000Z', finishedAt: '2026-01-01T00:00:45.000Z' }),
          job({ id: 'job-0', startedAt: '2026-01-01T00:00:00.000Z', finishedAt: '2026-01-01T00:05:00.000Z' }),
        ]}
      />,
    );
    expect(screen.getByText('45s')).toBeInTheDocument();
  });

  it('shows a dash when the most recent job never started', () => {
    renderWithI18n(<PipelineLastJob jobs={[job({ startedAt: undefined, finishedAt: undefined })]} />);
    expect(screen.getByText('-')).toBeInTheDocument();
  });

  it('measures a still-running most-recent job against now', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-01-01T00:01:30.000Z'));

    renderWithI18n(
      <PipelineLastJob
        jobs={[job({ status: 'running', startedAt: '2026-01-01T00:00:00.000Z', finishedAt: undefined })]}
      />,
    );
    expect(screen.getByText('1m 30s')).toBeInTheDocument();

    vi.useRealTimers();
  });
});
