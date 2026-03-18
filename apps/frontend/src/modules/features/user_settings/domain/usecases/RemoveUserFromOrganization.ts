import type { UserSettingsRepository } from '@/modules/features/user_settings/domain/repository/UserSettingsRepository.ts';
import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';

export class RemoveUserFromOrganization {
  constructor(private readonly userSettingsRepository: UserSettingsRepository) {}

  public async execute(userId: string, organizationId: string): Promise<ScyllaResult<void>> {
    return this.userSettingsRepository.removeUserFromOrganization(userId, organizationId);
  }
}
