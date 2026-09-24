// @vitest-environment node
import { describe, it, expect } from 'vitest';
import type { TrailCrumb } from '@platform/routing';
import { breadcrumbsFor } from '../breadcrumbs.ts';

const translate = (message: { id: string; message?: string }) =>
  `t:${message.message ?? message.id}`;

describe('breadcrumbsFor', () => {
  it('translates the label and the detail, and keeps the highlight as it is', () => {
    const trail: TrailCrumb[] = [
      {
        breadcrumb: ({ pipelineName }) => ({
          label: { id: 'Pipeline' },
          highlight: pipelineName,
          detail: { id: 'Jobs' },
        }),
        pathname: '/acme/projects/p1/pipelines/pl1/jobs',
      },
    ];

    expect(breadcrumbsFor(trail, { pipelineName: 'Nightly' }, translate)).toEqual([
      {
        label: 't:Pipeline',
        highlight: 'Nightly',
        detail: 't:Jobs',
        pathname: '/acme/projects/p1/pipelines/pl1/jobs',
      },
    ]);
  });
});
