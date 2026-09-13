import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithI18n } from '@/test/render.tsx';
import { JobStatus } from './JobStatus';
import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';

const job = (overrides: Partial<JobEntity> = {}): JobEntity => ({
  id: 'job-1',
  pipelineId: 'pipeline-1',
  status: 'completed',
  nodeExecutions: [],
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

describe('JobStatus', () => {
  it('shows the status label', () => {
    renderWithI18n(<JobStatus job={job({ status: 'completed' })} />);
    expect(screen.getByText('Success')).toBeInTheDocument();
  });

  it('adds a "queued" hint only for a pending job', () => {
    renderWithI18n(<JobStatus job={job({ status: 'pending' })} />);
    expect(screen.getByText('queued — waiting for an agent')).toBeInTheDocument();
  });

  it('adds a "disconnected" hint only for an orphaned job', () => {
    renderWithI18n(<JobStatus job={job({ status: 'orphaned' })} />);
    expect(screen.getByText('agent disconnected mid-run')).toBeInTheDocument();
  });

  it('shows no extra hint for a running/completed/failed job', () => {
    for (const status of ['running', 'completed', 'failed', 'cancelled']) {
      const { unmount } = renderWithI18n(<JobStatus job={job({ status })} />);
      expect(screen.queryByText('queued — waiting for an agent')).not.toBeInTheDocument();
      expect(screen.queryByText('agent disconnected mid-run')).not.toBeInTheDocument();
      unmount();
    }
  });

  it('falls back to the pending config for an unrecognized status, without an extra hint', () => {
    renderWithI18n(<JobStatus job={job({ status: 'not-a-real-status' })} />);
    expect(screen.getByText('Pending')).toBeInTheDocument();
    // getStatusConfig falls back to 'pending' wholesale, but the hint is keyed
    // on job.status literally ('pending'), which this job doesn't have.
    expect(screen.queryByText('queued — waiting for an agent')).not.toBeInTheDocument();
  });
});
