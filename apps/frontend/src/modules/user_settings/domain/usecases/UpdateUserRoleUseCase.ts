import type { UserSettingsRepository } from '@/modules/user_settings/domain/repository/UserSettingsRepository.ts';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';

export class UpdateUserRoleUseCase {
  constructor(private readonly userSettingsRepository: UserSettingsRepository) {}

  public async execute(
    userId: string,
    organizationId: string,
    newRole: string,
  ): Promise<ScyllaResult<void>> {
    return this.userSettingsRepository.updateUserRole(userId, organizationId, newRole);
  }
}
