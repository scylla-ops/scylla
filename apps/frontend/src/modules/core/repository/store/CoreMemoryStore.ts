export interface CoreMemoryStore {
  getToken: () => string | null;
  setToken: (token: string) => void;
}
