import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';
import type { OrganizationUser } from '@/modules/user_settings/domain/models/OrganizationUser.ts';

export interface UserSettingsRepository {
  getOrganizationUsers(
    organizationId: string,
    page?: number,
    pageSize?: number,
  ): Promise<ScyllaResult<{ users: OrganizationUser[]; pagination: any }>>;
  addUserToOrganization(
    userId: string,
    organizationId: string,
    role: string,
  ): Promise<ScyllaResult<void>>;
  removeUserFromOrganization(userId: string, organizationId: string): Promise<ScyllaResult<void>>;
  updateUserRole(
    userId: string,
    organizationId: string,
    newRole: string,
  ): Promise<ScyllaResult<void>>;
}
