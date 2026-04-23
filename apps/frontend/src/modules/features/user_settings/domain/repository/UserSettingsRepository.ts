import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { OrganizationUser } from '@/modules/features/user_settings/domain/models/OrganizationUser.ts';
import type { User } from '@/modules/features/user_settings/domain/models/User.ts';

export interface UserSettingsRepository {
  getUser(userId: string): Promise<ScyllaResult<User>>;
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
