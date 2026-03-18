import type { UserSettingsRepository } from '@/modules/user_settings/domain/repository/UserSettingsRepository.ts';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';

export class AddUserToOrganizationUseCase {
  constructor(private readonly userSettingsRepository: UserSettingsRepository) {}

  public async execute(
    userId: string,
    organizationId: string,
    role: string,
  ): Promise<ScyllaResult<void>> {
    return this.userSettingsRepository.addUserToOrganization(userId, organizationId, role);
  }
}
