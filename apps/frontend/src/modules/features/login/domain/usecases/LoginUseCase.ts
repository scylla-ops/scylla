import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { LoginRepository } from '@/modules/features/login/domain/repository/LoginRepository.ts';

export class LoginUseCase {
  constructor(private readonly loginRepository: LoginRepository) {}
  public execute(login: string, password: string): Promise<ScyllaResult<void>> {
    return this.loginRepository.login(login, password);
  }
}
