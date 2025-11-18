export interface LoginMemoryStore {
  setToken(token: string): void;
  getToken(): string | null;
}
