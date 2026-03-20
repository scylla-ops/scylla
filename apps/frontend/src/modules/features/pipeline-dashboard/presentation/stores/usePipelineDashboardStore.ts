import { create } from 'zustand';

interface PipelineDashboardState {
  selectedPipelineIds: string[];
  // Actions
  selectPipeline: (id: string) => void;
  clearSelection: () => void;
}

export const usePipelineDashboardStore = create<PipelineDashboardState>(set => ({
  selectedPipelineIds: [],

  selectPipeline: (id: string) =>
    set(state => {
      const isSelected = state.selectedPipelineIds.includes(id);
      return {
        selectedPipelineIds: isSelected
          ? state.selectedPipelineIds.filter(pId => pId !== id)
          : [...state.selectedPipelineIds, id],
      };
    }),

  clearSelection: () => set({ selectedPipelineIds: [] }),
}));
