import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { LoginRepository } from '@/modules/features/login/domain/repository/login.repository.ts';

export class LoginUseCase {
  constructor(private readonly loginRepository: LoginRepository) {}
  public execute(login: string, password: string): Promise<ScyllaResult<void>> {
    return this.loginRepository.login(login, password);
  }
}
