import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import type { EffectivePermissionsEntity } from '@/modules/features/permission/domain/entities/effective-permissions.entity.ts';

interface ContextItem {
  id: string | null;
  name: string | null;
}

interface ContextStore {
  organization: ContextItem;
  setOrganization: (id: string | null, name: string | null) => void;

  project: ContextItem;
  setProject: (id: string | null, name: string | null) => void;

  pipeline: ContextItem;
  setPipeline: (id: string | null, name: string | null) => void;

  /**
   * Effective permissions of the signed-in user, loaded once at login and
   * refreshed when the active organization/project changes (see
   * `usePermissionSync`). `null` until the first load — gating hooks read this
   * synchronously instead of each calling the backend.
   */
  permissions: EffectivePermissionsEntity | null;
  setPermissions: (permissions: EffectivePermissionsEntity | null) => void;

  reset: () => void;
}

const initialState = {
  organization: { id: null, name: null } as ContextItem,
  project: { id: null, name: null } as ContextItem,
  pipeline: { id: null, name: null } as ContextItem,
  permissions: null as EffectivePermissionsEntity | null,
};

export const useContextStore = create<ContextStore>()(
  persist(
    set => ({
      ...initialState,
      setOrganization: (id, name) =>
        set({
          organization: { id, name },
          project: { id: null, name: null },
          pipeline: { id: null, name: null },
        }),
      setProject: (id, name) => set({ project: { id, name } }),
      setPipeline: (id, name) => set({ pipeline: { id, name } }),
      setPermissions: permissions => set({ permissions }),
      reset: () => set(initialState),
    }),
    {
      name: 'scylla-context',
      storage: createJSONStorage(() => localStorage),
      // Permissions are session state — never persisted, so every new session
      // starts unknown (denied) and re-fetches instead of trusting stale data.
      partialize: state => ({
        organization: state.organization,
        project: state.project,
        pipeline: state.pipeline,
      }),
    },
  ),
);
