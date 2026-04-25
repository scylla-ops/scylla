import type { UserRepository } from '@/modules/features/user/domain/repository/user.repository.ts';

export class DeleteUserUseCase {
  constructor(private readonly _repository: UserRepository) {}

  execute = (userId: string) => this._repository.delete(userId);
}

