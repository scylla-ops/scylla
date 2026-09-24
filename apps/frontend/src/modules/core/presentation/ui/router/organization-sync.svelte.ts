import { untrack } from 'svelte';
import { navigateTo, contextStore } from '@platform/context';
import { createQuery } from '@platform/query';
import { slugifyOrgName } from '@shared/utils/slug.ts';
import { organizationQueries } from '@/modules/features/organization';

/**
 * Makes the organization of the URL the active organization.
 *
 * It finds the organization whose slug is `organizationSlug`. When no
 * organization matches, it goes to the dashboard of the first organization.
 */
export const syncOrganization = (organizationSlug: string | undefined): void => {
  const organizations = createQuery(() => organizationQueries.mine());

  $effect(() => {
    const list = organizations.data;
    if (!organizationSlug || organizations.isLoading || !list) return;

    untrack(() => {
      const store = contextStore.getState();
      const match = list.find(organization => slugifyOrgName(organization.name) === organizationSlug);

      if (match) {
        if (match.id !== store.organization.id) store.setOrganization(match.id, match.name);
        return;
      }

      const fallback = list[0];
      if (!fallback) return;

      navigateTo(`/${slugifyOrgName(fallback.name)}/dashboard`, { replace: true });
      store.setOrganization(fallback.id, fallback.name);
    });
  });
};
