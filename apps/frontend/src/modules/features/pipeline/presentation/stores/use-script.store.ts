import { create } from 'zustand';

type ScriptState = {
  script: string;
  initialScript: string;
  setScript: (newScript: string) => void;
  setInitialScript: (newScript: string) => void;
};

export const useScriptStore = create<ScriptState>(set => ({
  script: '',
  initialScript: '',
  setScript: (newScript: string) => set({ script: newScript }),
  setInitialScript: (newScript: string) => set({ initialScript: newScript }),
}));
