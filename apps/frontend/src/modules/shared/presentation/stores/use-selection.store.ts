import { create } from 'zustand';

interface SelectionState {
  selectedIds: Record<string, string[]>;
  select: (key: string, id: string) => void;
  clearSelection: (key: string) => void;
  getSelectedIds: (key: string) => string[];
}

export const useSelectionStore = create<SelectionState>((set, get) => ({
  selectedIds: {},

  select: (key, id) =>
    set(state => {
      const current = state.selectedIds[key] ?? [];
      const isSelected = current.includes(id);
      return {
        selectedIds: {
          ...state.selectedIds,
          [key]: isSelected ? current.filter(i => i !== id) : [...current, id],
        },
      };
    }),

  clearSelection: (key) =>
    set(state => ({
      selectedIds: {
        ...state.selectedIds,
        [key]: [],
      },
    })),

  getSelectedIds: (key) => get().selectedIds[key] ?? [],
}));

