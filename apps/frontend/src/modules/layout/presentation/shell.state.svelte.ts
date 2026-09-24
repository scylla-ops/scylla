import { navigateTo, contextStore } from '@platform/context';
import { createMutation, createQuery } from '@platform/query';
import { toRune } from '@shared/presentation/stores/to-rune.svelte.ts';
import type { FormValues } from '@shared/presentation/ui';
import { slugifyOrgName } from '@shared/utils/slug.ts';
import { organizationMutations, organizationQueries } from '@/modules/features/organization';
import { syncMyPermissions } from '@/modules/features/roles';

/**
 * The state of the app shell.
 *
 * It loads the organizations of the user, and it keeps the permissions store in
 * sync with the active organization and project. Create it once, in `Layout`.
 */
export const createShellState = () => {
  const organizations = createQuery(() => organizationQueries.mine());
  const createOrganization = createMutation(() => organizationMutations.create());
  const context = toRune(contextStore);

  $effect(() => {
    const { organization, project } = context();
    syncMyPermissions(organization.id, project.id);
  });

  return {
    get isLoading() {
      return organizations.isLoading;
    },
    get hasOrganizations() {
      return (organizations.data?.length ?? 0) > 0;
    },
    get isCreating() {
      return createOrganization.isPending;
    },
    createFirstOrganization: ({ name, description }: FormValues<'name' | 'description'>) => {
      if (!name.trim()) return;

      createOrganization.mutate(
        { name, description: description.trim() || undefined },
        { onSuccess: () => navigateTo(`/${slugifyOrgName(name)}/users/me`) },
      );
    },
  };
};

export type ShellState = ReturnType<typeof createShellState>;
