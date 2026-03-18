import type { UserSettingsRepository } from '@/modules/user_settings/domain/repository/UserSettingsRepository.ts';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';
import type { OrganizationUser } from '@/modules/user_settings/domain/models/OrganizationUser.ts';

export class GetOrganizationUsersUseCase {
  constructor(private readonly userSettingsRepository: UserSettingsRepository) {}

  public async execute(
    organizationId: string,
    page: number = 1,
    pageSize: number = 10,
  ): Promise<ScyllaResult<{ users: OrganizationUser[]; pagination: any }>> {
    return this.userSettingsRepository.getOrganizationUsers(organizationId, page, pageSize);
  }
}
