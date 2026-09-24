import { untrack } from 'svelte';
import { currentPathname, navigateTo, contextStore } from '@platform/context';
import { createQuery } from '@platform/query';
import { toRune } from '@shared/presentation/stores/to-rune.svelte.ts';
import { slugifyOrgName } from '@shared/utils/slug.ts';
import { projectQueries } from '@/modules/features/project';

const isPipelinePath = (pathname: string): boolean =>
  pathname.includes('/edit/') || pathname.includes('/pipelines/');

/**
 * Removes stale context when the user opens a project page.
 *
 * When the project of the URL does not exist, it clears the project and the
 * pipeline and goes back to the project list. Outside the pipeline pages, it
 * clears the active pipeline.
 */
export const cleanContext = (projectId: string | undefined): void => {
  const pathname = currentPathname();
  const context = toRune(contextStore);
  const organizationId = $derived(context().organization.id);
  const projects = createQuery(() => projectQueries.byOrganization(organizationId));

  $effect(() => {
    const list = projects.data?.projects;
    if (projects.isLoading || !list || !projectId) return;

    untrack(() => {
      const store = contextStore.getState();

      if (!list.some(project => project.id === projectId)) {
        store.setProject(null, null);
        store.setPipeline(null, null);
        const name = store.organization.name;
        navigateTo(name ? `/${slugifyOrgName(name)}/projects` : '/', { replace: true });
        return;
      }

      if (store.pipeline.id && !isPipelinePath(pathname)) store.setPipeline(null, null);
    });
  });
};
