import type { CoreRepository } from '@core/domain/repository/CoreRepository.ts';
import type { CoreMemoryStore } from '@core/repository/store/CoreMemoryStore.ts';

export class CoreRepositoryImpl implements CoreRepository {
  constructor(private readonly coreMemoryStore: CoreMemoryStore) {}

  setToken(token: string): void {
    this.coreMemoryStore.setToken(token);
  }
  getToken(): string | null {
    return this.coreMemoryStore.getToken();
  }
}
