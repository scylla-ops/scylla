import { useEffect, useRef } from 'react';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import { useRefreshMyPermissions } from '@/modules/features/permission/presentation/hooks/use-refresh-my-permissions.ts';

/**
 * Keeps the context store's permissions in sync with the session. The backend
 * is only called when the sync key — signed-in user, active organization,
 * active project — actually changes: once at login, then on org/project
 * switches. Re-renders and effect replays (e.g. StrictMode) hit the key guard
 * and never re-fetch. Mount it **once** (in `Layout`); everything else reads
 * the store synchronously through `useAuthorization`.
 */
export const usePermissionSync = () => {
  const refresh = useRefreshMyPermissions();
  const orgId = useContextStore(state => state.organization.id);
  const projectId = useContextStore(state => state.project.id);
  const lastSyncedKey = useRef<string | null>(null);

  useEffect(() => {
    const userId = localStorage.getItem('userId') ?? '';
    const key = `${userId}/${orgId ?? ''}/${projectId ?? ''}`;
    if (lastSyncedKey.current === key) return; // nothing relevant changed
    lastSyncedKey.current = key;
    void refresh();
  }, [refresh, orgId, projectId]);
};
