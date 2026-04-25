import type { UserRepository } from '@/modules/features/user/domain/repository/user.repository.ts';

export class CreateUserUseCase {
  constructor(private readonly _repository: UserRepository) {}

  execute = (username: string, password: string) => this._repository.create(username, password);
}

