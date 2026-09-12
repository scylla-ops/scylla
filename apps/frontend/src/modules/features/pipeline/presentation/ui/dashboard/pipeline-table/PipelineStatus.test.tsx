import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { PipelineStatus } from './PipelineStatus';
import type { PipelineMetadata } from '@/modules/features/pipeline/domain/structs/pipeline.struct.ts';

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

describe('PipelineStatus', () => {
  it("shows the pipeline's name, creation day, and (copyable) id", () => {
    const pipeline: PipelineMetadata = {
      id: 'pipeline-123',
      projectId: 'project-1',
      name: 'ci-pipeline',
      nodeCount: 3,
      createdAt: '2026-03-15T00:00:00.000Z',
      updatedAt: '2026-03-15T00:00:00.000Z',
    };
    renderWithI18n(<PipelineStatus pipeline={pipeline} status='success' />);

    expect(screen.getByText('ci-pipeline')).toBeInTheDocument();
    expect(screen.getByText('pipeline-123')).toBeInTheDocument();
    expect(screen.getByText(/Creation:/)).toBeInTheDocument();
  });
});
