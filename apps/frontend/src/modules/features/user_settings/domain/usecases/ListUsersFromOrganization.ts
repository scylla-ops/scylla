import type { UserSettingsRepository } from '@/modules/features/user_settings/domain/repository/UserSettingsRepository.ts';

export class ListUsersFromOrganization {
  constructor(private readonly _repository: UserSettingsRepository) {}

  public execute(organizationId: string, page: number = 1, pageSize: number = 10) {
    return this._repository.getOrganizationUsers(organizationId, page, pageSize);
  }
}
