import type { ScyllaResult } from '@/modules/core/domain/ScyllaResult.ts';
import type { LoginRepository } from '@/modules/login/domain/repository/LoginRepository.ts';

export class LoginUseCase {
  constructor(private readonly loginRepository: LoginRepository) {}
  public execute(login: string, password: string): Promise<ScyllaResult<string>> {
    return this.loginRepository.login(login, password);
  }
}
