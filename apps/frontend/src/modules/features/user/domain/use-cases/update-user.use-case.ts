import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { UserEntity } from '@/modules/features/user/domain/entities/user.entity.ts';
import type { UserRepository } from '@/modules/features/user/domain/repository/user.repository.ts';

export class UpdateUserUseCase {
  constructor(private readonly _userRepository: UserRepository) {}

  public async execute(userId: string, username?: string): Promise<ScyllaResult<UserEntity>> {
    return this._userRepository.update(userId, username);
  }
}
