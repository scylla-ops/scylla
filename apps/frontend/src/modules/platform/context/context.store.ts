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

/**
 * Which organization/project/pipeline the user is currently looking at.
 *
 * Holds identifiers only. The effective permissions of the user are in
 * `permissionsStore` (`@platform/authz`). Persisted in `localStorage`, so a
 * reload keeps the user in the same place.
 */
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
