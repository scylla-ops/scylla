import { create } from 'zustand/react';

//TODO: maybe move this into core, since it will be used be every module to call api
// (OR FIND ANOTHER BETTER SOLUTION TO STORE THE TOKEN)
interface TokenStore {
  token: string;
  setToken: (token: string) => void;
}
export const useToken = create<TokenStore>(set => ({
  token: '',
  setToken: (token: string) => set({ token }),
}));
