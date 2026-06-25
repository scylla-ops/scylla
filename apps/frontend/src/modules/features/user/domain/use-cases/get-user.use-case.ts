import type { UserRepository } from '@/modules/features/user/domain/repository/user.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { UserEntity } from '@/modules/features/user/domain/entities/user.entity.ts';

export class GetUserUseCase {
  constructor(private userRepository: UserRepository) {}

  public async execute(userId: string): Promise<ScyllaResult<UserEntity>> {
    return await this.userRepository.getById(userId);
  }
}
