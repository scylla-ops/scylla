import { create } from 'zustand';

interface JobsState {
  selectedJobIds: string[];
  expandedJobId: string | null;
  // Actions
  selectJob: (id: string) => void;
  clearSelection: () => void;
  toggleExpand: (id: string | null) => void;
}

export const useJobsStore = create<JobsState>(set => ({
  selectedJobIds: [],
  expandedJobId: null,

  selectJob: (id: string) =>
    set(state => {
      const isSelected = state.selectedJobIds.includes(id);
      return {
        selectedJobIds: isSelected
          ? state.selectedJobIds.filter(jobId => jobId !== id)
          : [...state.selectedJobIds, id],
      };
    }),

  clearSelection: () => set({ selectedJobIds: [] }),

  toggleExpand: (id: string | null) =>
    set(state => ({
      expandedJobId: state.expandedJobId === id ? null : id,
    })),
}));

