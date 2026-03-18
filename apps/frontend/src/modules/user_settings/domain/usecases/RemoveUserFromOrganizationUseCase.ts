import type { UserSettingsRepository } from '@/modules/user_settings/domain/repository/UserSettingsRepository.ts';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';

export class RemoveUserFromOrganizationUseCase {
  constructor(private readonly userSettingsRepository: UserSettingsRepository) {}

  public async execute(userId: string, organizationId: string): Promise<ScyllaResult<void>> {
    return this.userSettingsRepository.removeUserFromOrganization(userId, organizationId);
  }
}
