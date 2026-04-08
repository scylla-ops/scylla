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
}

export const useContextStore = create<ContextStore>()(
  persist(
    set => ({
      organization: { id: null, name: null },
      setOrganization: (id, name) => set({ organization: { id, name } }),

      project: { id: null, name: null },
      setProject: (id, name) => set({ project: { id, name } }),

      pipeline: { id: null, name: null },
      setPipeline: (id, name) => set({ pipeline: { id, name } }),
    }),
    {
      name: 'scylla-context-storage',
      storage: createJSONStorage(() => localStorage),
    },
  ),
);
