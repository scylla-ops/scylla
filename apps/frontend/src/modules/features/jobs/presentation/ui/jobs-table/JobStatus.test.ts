import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/svelte';
import { render } from '@/test/render.svelte.ts';
import type { JobEntity } from '../../../domain/entities/job.entity.ts';
import JobStatus from './JobStatus.svelte';

const job = (status: string): JobEntity => ({
  id: 'job-1',
  pipelineId: 'pipeline-1',
  status,
  nodeExecutions: [],
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:01:00.000Z',
});

describe('JobStatus', () => {
  it('shows the status label', () => {
    render(JobStatus, { job: job('completed') });
    expect(screen.getByText('Success')).toBeInTheDocument();
  });

  it('adds a "queued" hint only for a pending job', () => {
    render(JobStatus, { job: job('pending') });
    expect(screen.getByText('queued — waiting for an agent')).toBeInTheDocument();
  });

  it('adds a "disconnected" hint only for an orphaned job', () => {
    render(JobStatus, { job: job('orphaned') });
    expect(screen.getByText('agent disconnected mid-run')).toBeInTheDocument();
  });

  it('shows no extra hint for a running, completed or failed job', () => {
    for (const status of ['running', 'completed', 'failed']) {
      const { unmount } = render(JobStatus, { job: job(status) });

      expect(screen.queryByText('queued — waiting for an agent')).not.toBeInTheDocument();
      expect(screen.queryByText('agent disconnected mid-run')).not.toBeInTheDocument();

      unmount();
    }
  });

  it('falls back to the pending config for an unrecognized status, without its hint', () => {
    // The fallback is about *colour and label*, not about the job being queued
    // — a status this build does not know is not a promise that it is waiting.
    render(JobStatus, { job: job('teleported') });

    expect(screen.getByText('Pending')).toBeInTheDocument();
    expect(screen.queryByText('queued — waiting for an agent')).not.toBeInTheDocument();
  });
});
