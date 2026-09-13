import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithI18n } from '@/test/render.tsx';
import userEvent from '@testing-library/user-event';
import { JobLogDialog } from './JobLogDialog';

vi.mock('@/modules/features/jobs/presentation/ui/jobs-table/jobs-log/JobLogDisplay.tsx', () => ({
  JobLogDisplay: ({ jobId, nodeId }: { jobId: string; nodeId?: string }) => (
    <div data-testid='job-log-display'>
      logs for {jobId}/{nodeId ?? 'whole job'}
    </div>
  ),
}));

describe('JobLogDialog', () => {
  it('stays closed, rendering nothing, without a jobId', () => {
    renderWithI18n(<JobLogDialog jobId={undefined} onClose={vi.fn()} />);
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('opens once a jobId is given, showing the job id badge', () => {
    renderWithI18n(<JobLogDialog jobId='job-42' onClose={vi.fn()} />);
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(screen.getByText('#job-42')).toBeInTheDocument();
    expect(screen.queryByText(/Node #/)).not.toBeInTheDocument();
    expect(screen.getByText('logs for job-42/whole job')).toBeInTheDocument();
  });

  it('shows a node badge too, and scopes the log view to it, when a nodeId is given', () => {
    renderWithI18n(<JobLogDialog jobId='job-42' nodeId='build' onClose={vi.fn()} />);
    expect(screen.getByText('Node #build')).toBeInTheDocument();
    expect(screen.getByText('logs for job-42/build')).toBeInTheDocument();
  });

  it('closing the dialog calls onClose', async () => {
    const onClose = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<JobLogDialog jobId='job-42' onClose={onClose} />);

    await user.keyboard('{Escape}');
    expect(onClose).toHaveBeenCalled();
  });
});
