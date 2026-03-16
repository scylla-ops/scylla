import { create } from 'zustand';

interface OrganizationStore {
  currentOrganizationName: string;
  setCurrentOrganizationName: (name: string) => void;
}

export const useOrganizationStore = create<OrganizationStore>(set => ({
  currentOrganizationName: 'Select an organization',
  setCurrentOrganizationName: name => set({ currentOrganizationName: name }),
}));
