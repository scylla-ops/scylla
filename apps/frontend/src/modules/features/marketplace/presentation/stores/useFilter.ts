import { create } from 'zustand';

type FilterState = {
  filter: string;
  setFilter: (newFilter: string) => void;
};

export const useFilterStore = create<FilterState>(set => ({
  filter: '',
  setFilter: (newFilter: string) => set({ filter: newFilter }),
}));
