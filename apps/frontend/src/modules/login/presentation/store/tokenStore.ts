import { create } from 'zustand/react';

interface TokenStore {
  token: string;
  setToken: (token: string) => void;
}
export const useToken = create<TokenStore>(set => ({
  token: '',
  setToken: (token: string) => set({ token }),
}));
