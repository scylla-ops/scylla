import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { flushSync } from 'svelte';
import { contextStore } from '@platform/context';
import { stubQuery } from '@/test/queries.ts';
import { installTestNavigator } from '@/test/navigator.ts';
import { withQueryClient } from '@/test/render.svelte.ts';
import { cleanContext } from '../context-cleaner.svelte.ts';

const projectsState: { projects?: { id: string; name: string }[]; isLoading: boolean } = {
  projects: undefined,
  isLoading: false,
};

vi.mock('@/modules/features/project', () => ({
  projectQueries: {
    byOrganization: () =>
      stubQuery(['projects'], projectsState.projects && { projects: projectsState.projects }, {
        loading: projectsState.isLoading,
      }),
  },
}));

let navigator: ReturnType<typeof installTestNavigator>;
let cache: ReturnType<typeof withQueryClient>;

const run = (projectId: string | undefined, pathname = '/acme/projects/project-1') => {
  navigator.restore();
  navigator = installTestNavigator({ pathname });
  const cleanup = $effect.root(() => {
    cleanContext(projectId);
  });
  flushSync();
  cleanup();
};

beforeEach(() => {
  navigator = installTestNavigator();
  cache = withQueryClient();
  projectsState.projects = [{ id: 'project-1', name: 'web' }];
  projectsState.isLoading = false;
  contextStore.setState({
    organization: { id: 'org-1', name: 'Acme Corp' },
    project: { id: 'project-1', name: 'web' },
    pipeline: { id: null, name: null },
  });
});

afterEach(() => {
  navigator.restore();
  cache.restore();
});

describe('cleanContext', () => {
  it('does nothing while the projects are still loading', () => {
    projectsState.isLoading = true;
    run('project-1');

    expect(navigator.navigate).not.toHaveBeenCalled();
  });

  it('does nothing without a project id', () => {
    run(undefined);

    expect(navigator.navigate).not.toHaveBeenCalled();
    expect(contextStore.getState().project.id).toBe('project-1');
  });

  it('leaves a project that still exists alone', () => {
    run('project-1');

    expect(navigator.navigate).not.toHaveBeenCalled();
    expect(contextStore.getState().project).toEqual({ id: 'project-1', name: 'web' });
  });

  it('clears the project and the pipeline and goes to the project list when the project is gone', () => {
    run('deleted-project');

    expect(contextStore.getState().project).toEqual({ id: null, name: null });
    expect(contextStore.getState().pipeline).toEqual({ id: null, name: null });
    expect(navigator.navigate).toHaveBeenCalledWith('/acme-corp/projects', { replace: true });
  });

  it('goes to "/" when there is no organization name to make a slug from', () => {
    contextStore.setState({ organization: { id: 'org-1', name: null } });
    run('deleted-project');

    expect(navigator.navigate).toHaveBeenCalledWith('/', { replace: true });
  });

  it('clears a stale pipeline outside the pipeline pages', () => {
    contextStore.setState({ pipeline: { id: 'pipeline-1', name: 'ci' } });
    run('project-1', '/acme/projects/project-1/settings');

    expect(contextStore.getState().pipeline).toEqual({ id: null, name: null });
  });

  it('keeps the pipeline on its editor page', () => {
    contextStore.setState({ pipeline: { id: 'pipeline-1', name: 'ci' } });
    run('project-1', '/acme/projects/project-1/edit/pipeline-1');

    expect(contextStore.getState().pipeline).toEqual({ id: 'pipeline-1', name: 'ci' });
  });

  it('keeps the pipeline on any /pipelines/ page', () => {
    contextStore.setState({ pipeline: { id: 'pipeline-1', name: 'ci' } });
    run('project-1', '/acme/projects/project-1/pipelines/pipeline-1/jobs');

    expect(contextStore.getState().pipeline).toEqual({ id: 'pipeline-1', name: 'ci' });
  });
});
