import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithI18n } from '@/test/render.tsx';
import { JobIdCell } from './JobIdCell';
import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';

describe('JobIdCell', () => {
  it('truncates the id to 12 characters with an ellipsis', () => {
    const job = { id: 'job-123456789012345' } as JobEntity;
    renderWithI18n(<JobIdCell job={job} />);
    // CopyableText itself is covered elsewhere; this just confirms the id and
    // the 12-char truncation are wired through to it.
    expect(screen.getByText('job-12345678...')).toBeInTheDocument();
  });
});
