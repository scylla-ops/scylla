import type { UserSettingsRepository } from '@/modules/features/user_settings/domain/repository/UserSettingsRepository.ts';
import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';

export class UpdateUserRole {
  constructor(private readonly userSettingsRepository: UserSettingsRepository) {}

  public async execute(
    userId: string,
    organizationId: string,
    newRole: string,
  ): Promise<ScyllaResult<void>> {
    return this.userSettingsRepository.updateUserRole(userId, organizationId, newRole);
  }
}
