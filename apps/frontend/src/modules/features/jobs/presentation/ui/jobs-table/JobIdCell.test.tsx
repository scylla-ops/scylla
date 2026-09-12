import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { JobIdCell } from './JobIdCell';
import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

beforeEach(() => {
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
});

describe('JobIdCell', () => {
  it('truncates the id to 12 characters with an ellipsis', () => {
    const job = { id: 'job-123456789012345' } as JobEntity;
    renderWithI18n(<JobIdCell job={job} />);
    // CopyableText itself is covered elsewhere; this just confirms the id and
    // the 12-char truncation are wired through to it.
    expect(screen.getByText('job-12345678...')).toBeInTheDocument();
  });
});
