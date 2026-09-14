import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithI18n } from '@/test/render.tsx';
import { PipelineStatus } from './PipelineStatus';
import type { PipelineMetadata } from '@/modules/features/pipeline/domain/structs/pipeline.struct.ts';

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
