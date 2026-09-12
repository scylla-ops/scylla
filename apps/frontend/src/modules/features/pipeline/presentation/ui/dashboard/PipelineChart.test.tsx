import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { PipelineChart } from './PipelineChart';
import type { JobEntity } from '@/modules/features/jobs';

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

const job = (overrides: Partial<JobEntity> = {}): JobEntity => ({
  id: 'job-1',
  pipelineId: 'p1',
  status: 'completed',
  nodeExecutions: [],
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:01:00.000Z',
  ...overrides,
});

describe('PipelineChart', () => {
  it('shows skeleton placeholders while loading', () => {
    const { container } = renderWithI18n(<PipelineChart jobs={[]} isLoading maxJobs={5} />);
    expect(container.querySelectorAll('[data-slot="skeleton"]')).toHaveLength(5);
  });

  it('says the caller lacks permission when the history is forbidden - checked before the error state', () => {
    renderWithI18n(<PipelineChart jobs={[]} isForbidden isError />);
    expect(screen.getByText("You don't have permission to view this pipeline's jobs")).toBeInTheDocument();
  });

  it('shows an error message when loading failed', () => {
    renderWithI18n(<PipelineChart jobs={[]} isError />);
    expect(screen.getByText('Error loading jobs')).toBeInTheDocument();
  });

  it('shows an empty-history placeholder with no jobs at all', () => {
    renderWithI18n(<PipelineChart jobs={[]} />);
    expect(screen.getByText('No jobs yet')).toBeInTheDocument();
  });

  it('caps the number of segments shown at maxJobs', () => {
    const jobs = Array.from({ length: 15 }, (_, i) => job({ id: `job-${i}` }));
    const { container } = renderWithI18n(<PipelineChart jobs={jobs} maxJobs={10} />);
    expect(container.querySelectorAll('.flex-1.min-w-\\[2px\\]')).toHaveLength(10);
  });
});
