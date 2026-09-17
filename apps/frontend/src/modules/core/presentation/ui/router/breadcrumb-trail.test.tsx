import { describe, it, expect, beforeEach } from 'vitest';
import { screen } from '@testing-library/react';
import { createMemoryRouter, Outlet, RouterProvider, type RouteObject } from 'react-router-dom';
import type { MessageDescriptor } from '@lingui/core';
import { renderWithI18n } from '@/test/render.tsx';
import { routesFor, type BreadcrumbParams } from '@platform/routing';
import { useContextStore } from '@platform/context';
import { modules } from '@core/di/registry.ts';
import { ScyllaBreadcrumbs } from '@/modules/layout/presentation/ui/ScyllaBreadcrumbs.tsx';

interface LooseRoute {
  path?: string;
  index?: boolean;
  handle?: { breadcrumb?: (params: BreadcrumbParams) => unknown };
  lazy?: unknown;
  children?: LooseRoute[];
}

/**
 * The real routes, minus the chunks: `lazy` would pull every page — and the
 * providers they expect — into a test about the trail above them. The crumbs
 * come from the handles, which stay, so a blank stand-in page is enough.
 */
const withoutPages = (routes: readonly LooseRoute[]): RouteObject[] =>
  routes.map(({ lazy: _lazy, children, ...route }) => ({
    ...route,
    ...(children ? { children: withoutPages(children) } : { element: <div /> }),
  })) as RouteObject[];

const label = (id: string): MessageDescriptor => ({ id });

/**
 * `Core.router` mirrored down to where it matters: the breadcrumbs render in
 * the shell, above every path segment, and the two outermost crumbs are the
 * shell's own. What hangs below is the modules' real composition.
 */
const renderTrailAt = (path: string) => {
  const router = createMemoryRouter(
    [
      {
        element: (
          <>
            <ScyllaBreadcrumbs />
            <Outlet />
          </>
        ),
        children: [
          {
            path: '/:organizationSlug',
            children: [
              {
                path: 'projects',
                handle: { breadcrumb: () => ({ label: label('Projects') }) },
                children: [
                  {
                    path: ':projectId',
                    handle: {
                      breadcrumb: ({ projectName }: BreadcrumbParams) => ({
                        label: label('Project'),
                        highlight: projectName,
                      }),
                    },
                    children: withoutPages(routesFor(modules, 'project')),
                  },
                ],
              },
            ],
          },
        ],
      },
    ],
    { initialEntries: [path] },
  );

  return renderWithI18n(<RouterProvider router={router} />);
};

const trail = () => screen.getAllByRole('listitem').map(item => item.textContent);

const jobsPath = '/acme/projects/project-1/pipelines/pipeline-1/jobs';
const jobPath = `${jobsPath}/job-42`;

beforeEach(() => {
  useContextStore.setState({
    organization: { id: 'org-1', name: 'Acme' },
    project: { id: 'project-1', name: 'Scylla' },
    pipeline: { id: 'pipeline-1', name: 'Nightly' },
  });
});

describe('the breadcrumb trail the modules compose', () => {
  it('ends on the pipeline\'s jobs for the list', () => {
    renderTrailAt(jobsPath);

    expect(trail()).toEqual(['Projects', 'Project#Scylla', 'Pipeline#Nightly- Jobs']);
  });

  it('keeps that crumb and adds the job for one job\'s page', () => {
    renderTrailAt(jobPath);

    expect(trail()).toEqual([
      'Projects',
      'Project#Scylla',
      'Pipeline#Nightly- Jobs',
      'Job#job-42',
    ]);
  });

  it('leaves the jobs crumb clickable and the job itself the current page', () => {
    renderTrailAt(jobPath);

    expect(screen.getByRole('link', { name: /Jobs/ })).toHaveAttribute('href', jobsPath);
    expect(screen.getByRole('link', { name: /job-42/ })).toHaveAttribute('aria-current', 'page');
  });
});
