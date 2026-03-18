import type { UserSettingsRepository } from '@/modules/features/user_settings/domain/repository/UserSettingsRepository.ts';
import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { OrganizationUser } from '@/modules/features/user_settings/domain/models/OrganizationUser.ts';

export class GetOrganizationUsers {
  constructor(private readonly userSettingsRepository: UserSettingsRepository) {}

  public async execute(
    organizationId: string,
    page: number = 1,
    pageSize: number = 10,
  ): Promise<ScyllaResult<{ users: OrganizationUser[]; pagination: any }>> {
    return this.userSettingsRepository.getOrganizationUsers(organizationId, page, pageSize);
  }
}
