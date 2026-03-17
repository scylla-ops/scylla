import { create } from 'zustand';

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
}

export const useContextStore = create<ContextStore>(set => ({
  organization: {
    id: null,
    name: null,
  },
  setOrganization: (id, name) =>
    set({
      organization: { id, name },
    }),

  project: {
    id: null,
    name: null,
  },
  setProject: (id, name) =>
    set({
      project: { id, name },
    }),
}));
