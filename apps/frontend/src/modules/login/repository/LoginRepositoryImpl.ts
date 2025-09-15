import type { LoginRepository } from '@/modules/login/domain/repository/LoginRepository.ts';
import type { ScyllaResult } from '@/modules/core/domain/ScyllaResult.ts';

export class LoginRepositoryImpl implements LoginRepository {
  constructor(private readonly loginRepository: LoginRepository) {}

  login(username: string, password: string): Promise<ScyllaResult<void>> {
    return this.loginRepository.login(username, password);
  }
}
