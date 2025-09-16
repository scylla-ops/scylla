import type { LoginRepository } from '@/modules/login/domain/repository/LoginRepository.ts';
import type { ScyllaResult } from '@/modules/core/domain/ScyllaResult.ts';
import type { LoginStore } from '@/modules/login/repository/store/LoginStore.ts';

export class LoginRepositoryImpl implements LoginRepository {
  constructor(private readonly loginStore: LoginStore) {}

  login(username: string, password: string): Promise<ScyllaResult<string>> {
    return this.loginStore.login(username, password);
  }
}
