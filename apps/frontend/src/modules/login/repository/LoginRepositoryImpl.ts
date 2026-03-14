import type { LoginRepository } from '@/modules/login/domain/repository/LoginRepository.ts';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';
import type { LoginRemoteDataSource } from '@/modules/login/repository/data-sources/LoginRemoteDataSource.ts';

export class LoginRepositoryImpl implements LoginRepository {
  constructor(private readonly loginRemoteStore: LoginRemoteDataSource) {}

  login(username: string, password: string): Promise<ScyllaResult<void>> {
    return this.loginRemoteStore.login(username, password);
  }
}
