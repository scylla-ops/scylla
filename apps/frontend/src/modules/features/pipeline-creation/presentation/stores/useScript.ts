import { create } from 'zustand';

type ScriptState = {
  script: string;
  setScript: (newScript: string) => void;
};

export const useScriptStore = create<ScriptState>(set => ({
  script: '',
  setScript: (newScript: string) => set({ script: newScript }),
}));
