import type { LoginRepository } from '@/modules/features/login/domain/repository/login.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { LoginRemoteDataSource } from '@/modules/features/login/infrastructure/repository/data-sources/login-remote.data-source.ts';

export class DefaultLoginRepository implements LoginRepository {
  constructor(private readonly loginRemoteStore: LoginRemoteDataSource) {}

  login(username: string, password: string): Promise<ScyllaResult<void>> {
    return this.loginRemoteStore.login(username, password);
  }
}
