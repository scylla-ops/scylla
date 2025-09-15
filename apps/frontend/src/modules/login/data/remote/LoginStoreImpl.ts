import type { LoginStore } from '@/modules/login/repository/store/LoginStore.ts';
import type { ScyllaResult } from '@/modules/core/domain/ScyllaResult.ts';

export class LoginStoreImpl implements LoginStore {
  login(username: string, password: string): Promise<ScyllaResult<void>> {
    return Promise.resolve(undefined);
  }
}
