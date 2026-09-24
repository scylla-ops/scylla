import { navigateTo, contextStore } from '@platform/context';
import { createQuery } from '@platform/query';
import { toRune } from '@shared/presentation/stores/to-rune.svelte.ts';
import { slugifyOrgName } from '@shared/utils/slug.ts';
import { organizationQueries } from '@/modules/features/organization';

/**
 * Sends the user to the dashboard of the active organization, or of the first
 * organization. With no organization, it stays: the layout shows the welcome
 * screen.
 */
export const redirectToOrganization = (): void => {
  const organizations = createQuery(() => organizationQueries.mine());
  const context = toRune(contextStore);
  let done = false;

  $effect(() => {
    if (done || organizations.isLoading) return;

    const name = context().organization.name ?? organizations.data?.[0]?.name;
    if (!name) return;

    done = true;
    navigateTo(`/${slugifyOrgName(name)}/dashboard`, { replace: true });
  });
};
