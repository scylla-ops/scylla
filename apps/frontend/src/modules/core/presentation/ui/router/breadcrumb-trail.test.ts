import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { contextStore, navigateTo, setAppNavigator } from '@platform/context';
import { createAppRouter } from '@platform/routing';
import ScyllaBreadcrumbs from '@/modules/layout/presentation/ui/ScyllaBreadcrumbs.svelte';
import { appRoutes } from './core.router.ts';

const jobsPath = '/acme/projects/project-1/pipelines/pipeline-1/jobs';
const jobPath = `${jobsPath}/job-42`;

const trail = () => screen.getAllByRole('listitem').map(item => item.textContent?.trim());

const renderTrailAt = async (path: string) => {
  setAppNavigator(createAppRouter(appRoutes));
  navigateTo(path);
  render(ScyllaBreadcrumbs);
  await expect.poll(() => window.location.pathname).toBe(path);
};

beforeEach(() => {
  contextStore.setState({
    organization: { id: 'org-1', name: 'Acme' },
    project: { id: 'project-1', name: 'Scylla' },
    pipeline: { id: 'pipeline-1', name: 'Nightly' },
  });
});

afterEach(() => setAppNavigator(null));

describe('the breadcrumb trail the modules compose', () => {
  it("ends on the pipeline's jobs for the list", async () => {
    await renderTrailAt(jobsPath);

    expect(trail()).toEqual(['Projects', 'Project#Scylla', 'Pipeline#Nightly- Jobs']);
  });

  it("keeps that crumb and adds the job for one job's page", async () => {
    await renderTrailAt(jobPath);

    expect(trail()).toEqual(['Projects', 'Project#Scylla', 'Pipeline#Nightly- Jobs', 'Job#job-42']);
  });

  it('leaves the jobs crumb clickable and the job itself the current page', async () => {
    await renderTrailAt(jobPath);

    expect(screen.getByRole('link', { name: /Jobs/ })).toHaveAttribute('href', jobsPath);
    expect(screen.getByRole('link', { name: /job-42/ })).toHaveAttribute('aria-current', 'page');
  });
});
