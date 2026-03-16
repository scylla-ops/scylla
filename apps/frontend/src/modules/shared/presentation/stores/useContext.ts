import { create } from 'zustand';

interface ContextStore {
  organizationId: string | null;
  setOrganizationId: (id: string) => void;

  projectId: string | null;
  setProjectId: (id: string) => void;
}

export const useContextStore = create<ContextStore>(set => ({
  organizationId: null,
  setOrganizationId: id => set({ organizationId: id }),
  projectId: null,
  setProjectId: id => set({ projectId: id }),
}));
