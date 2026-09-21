import { useEffect } from 'react';
import { useContextStore } from '@platform/context';
import { syncMyPermissions } from '@/modules/features/roles';

/**
 * Keeps the permissions store in sync with the session, from the React shell.
 *
 * The React binding of `syncMyPermissions`, which owns the rule: the backend is
 * called only when the sync key — signed-in user, active organization, active
 * project — actually changes, so re-renders and effect replays (StrictMode) hit
 * the guard and never re-fetch.
 *
 * The binding lives here, in the shell, rather than in `roles`: that module went
 * Svelte in Phase 4 and holds no React. Phase 6 replaces this file with a Svelte
 * effect and `syncMyPermissions` does not change. Mount it **once** — everything
 * else reads the store synchronously through `can()`.
 */
export const usePermissionSync = (): void => {
  const organizationId = useContextStore(state => state.organization.id);
  const projectId = useContextStore(state => state.project.id);

  useEffect(() => {
    syncMyPermissions(organizationId, projectId);
  }, [organizationId, projectId]);
};
