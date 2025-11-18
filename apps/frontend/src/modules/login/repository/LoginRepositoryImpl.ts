import type { LoginRepository } from '@/modules/login/domain/repository/LoginRepository.ts';
import type { ScyllaResult } from '@/modules/core/domain/ScyllaResult.ts';
import type { LoginRemoteStore } from '@/modules/login/repository/store/LoginRemoteStore.ts';
import type { LoginMemoryStore } from '@/modules/login/repository/store/LoginMemoryStore.ts';

export class LoginRepositoryImpl implements LoginRepository {
  constructor(private readonly loginRemoteStore: LoginRemoteStore) {}

  login(username: string, password: string): Promise<ScyllaResult<string>> {
    return this.loginRemoteStore.login(username, password);
  }
}
