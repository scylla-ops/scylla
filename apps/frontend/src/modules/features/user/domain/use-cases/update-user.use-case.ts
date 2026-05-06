import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { User } from '@/modules/features/user/domain/models/user.model.ts';
import type { UserRepository } from '@/modules/features/user/domain/repository/user.repository.ts';

export class UpdateUserUseCase {
  constructor(private readonly _userRepository: UserRepository) {}

  public async execute(userId: string, username?: string): Promise<ScyllaResult<User>> {
    return this._userRepository.update(userId, username);
  }
}
