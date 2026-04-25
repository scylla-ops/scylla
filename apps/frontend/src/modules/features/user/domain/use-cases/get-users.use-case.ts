import type { UserRepository } from '@/modules/features/user/domain/repository/user.repository.ts';

export class GetUsersUseCase {
  constructor(private readonly _repository: UserRepository) {}

  execute = () => this._repository.getAll();
}
