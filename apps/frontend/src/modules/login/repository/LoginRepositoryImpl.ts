import type { LoginRepository } from '@/modules/login/domain/repository/LoginRepository.ts';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';
import type { LoginRemoteStore } from '@/modules/login/repository/store/LoginRemoteStore.ts';

export class LoginRepositoryImpl implements LoginRepository {
  constructor(private readonly loginRemoteStore: LoginRemoteStore) {}

  login(username: string, password: string): Promise<ScyllaResult<void>> {
    return this.loginRemoteStore.login(username, password);
  }
}
