import type { UserRepository } from '@/modules/features/user/domain/repository/user.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { User } from '@/modules/features/user/domain/models/user.model.ts';

export class GetUserUseCase {
  constructor(private userRepository: UserRepository) {}

  public async execute(userId: string): Promise<ScyllaResult<User>> {
    return await this.userRepository.getById(userId);
  }
}
