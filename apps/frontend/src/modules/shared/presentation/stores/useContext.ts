import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface ContextStore {
  organization: {
    id: string | null;
    name: string | null;
  };
  setOrganization: (id: string, name: string) => void;

  project: {
    id: string | null;
    name: string | null;
  };
  setProject: (id: string, name: string) => void;

  reset: () => void;
}

const initialState = {
  organization: { id: null, name: null } as { id: string | null; name: string | null },
  project: { id: null, name: null } as { id: string | null; name: string | null },
};

export const useContextStore = create<ContextStore>()(
  persist(
    set => ({
      ...initialState,
      setOrganization: (id, name) =>
        set({ organization: { id, name }, project: { id: null, name: null } }),
      setProject: (id, name) => set({ project: { id, name } }),
      reset: () => set(initialState),
    }),
    { name: 'scylla-context' },
  ),
);
