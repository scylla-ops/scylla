import { createStore } from '@shared/presentation/stores/create-store.ts';

interface ContextItem {
  id: string | null;
  name: string | null;
}

interface ContextState {
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

/** Ids and names only, persisted in `localStorage`. */
export const contextStore = createStore<ContextState>(
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
  { persistAs: 'scylla-context' },
);
