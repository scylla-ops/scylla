import type { CoreMemoryStore } from '@core/repository/store/CoreMemoryStore.ts';

export class CoreMemoryStoreImpl implements CoreMemoryStore {
  private _token: string | null = null;

  getToken(): string | null {
    return this._token;
  }

  setToken(token: string): void {
    this._token = token;
  }
}
