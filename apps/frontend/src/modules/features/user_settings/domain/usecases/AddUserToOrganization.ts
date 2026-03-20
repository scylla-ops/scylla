import type { UserSettingsRepository } from '@/modules/features/user_settings/domain/repository/UserSettingsRepository.ts';
import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';

export class AddUserToOrganization {
  constructor(private readonly userSettingsRepository: UserSettingsRepository) {}

  public async execute(
    userId: string,
    organizationId: string,
    role: string,
  ): Promise<ScyllaResult<void>> {
    return this.userSettingsRepository.addUserToOrganization(userId, organizationId, role);
  }
}
