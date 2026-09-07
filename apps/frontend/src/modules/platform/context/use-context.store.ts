import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';

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

  reset: () => void;
}

const initialState = {
  organization: { id: null, name: null } as ContextItem,
  project: { id: null, name: null } as ContextItem,
  pipeline: { id: null, name: null } as ContextItem,
};

/**
 * Which organization/project/pipeline the user is currently looking at.
 *
 * Holds identifiers only. The user's effective permissions used to live here
 * too, which made this generic store depend on the permission feature; they now
 * belong to `usePermissionsStore`.
 */
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
      reset: () => set(initialState),
    }),
    {
      name: 'scylla-context',
      storage: createJSONStorage(() => localStorage),
    },
  ),
);
