import { create } from 'zustand';

interface ProjectStore {
  currentProjectName: string;
  setCurrentProjectName: (name: string) => void;
}

export const useProjectStore = create<ProjectStore>(set => ({
  currentProjectName: 'Select a Project',
  setCurrentProjectName: name => set({ currentProjectName: name }),
}));
