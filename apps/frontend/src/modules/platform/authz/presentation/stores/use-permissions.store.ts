import { create } from 'zustand';
import type { EffectivePermissionsEntity } from '@platform/authz/domain/entities/effective-permissions.entity.ts';

interface PermissionsStore {
  /**
   * Effective permissions of the signed-in user, loaded once at login and
   * refreshed when the active organization/project changes (see
   * `usePermissionSync`). `null` until the first load — gating hooks read this
   * synchronously instead of each calling the backend.
   */
  permissions: EffectivePermissionsEntity | null;
  setPermissions: (permissions: EffectivePermissionsEntity | null) => void;
}

/**
 * Deliberately not persisted: permissions are session state, so every new
 * session starts unknown (denied) and re-fetches instead of trusting a stale
 * copy from localStorage.
 */
export const usePermissionsStore = create<PermissionsStore>()(set => ({
  permissions: null,
  setPermissions: permissions => set({ permissions }),
}));
