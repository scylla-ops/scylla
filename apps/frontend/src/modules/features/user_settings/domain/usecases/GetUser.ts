import type { UserSettingsRepository } from '@/modules/features/user_settings/domain/repository/UserSettingsRepository.ts';
import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { User } from '@/modules/features/user_settings/domain/models/User.ts';

export class GetUser {
  constructor(private readonly userSettingsRepository: UserSettingsRepository) {}

  public async execute(userId: string): Promise<ScyllaResult<User>> {
    return this.userSettingsRepository.getUser(userId);
  }
}
