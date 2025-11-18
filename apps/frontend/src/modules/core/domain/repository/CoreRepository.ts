export interface CoreRepository {
  setToken: (token: string) => void;
  getToken: () => string | null;
}
