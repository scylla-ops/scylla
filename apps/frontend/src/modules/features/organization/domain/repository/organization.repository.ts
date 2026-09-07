import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { UserEntity } from '@/modules/features/user/domain/entities/user.entity.ts';
import type { OrganizationEntity } from '@/modules/features/organization/domain/entities/organization.entity.ts';

export interface OrganizationRepository {
  getAll(): Promise<ScyllaResult<OrganizationEntity[]>>;
  getMine(): Promise<ScyllaResult<OrganizationEntity[]>>;
  listMembers(organizationId: string): Promise<ScyllaResult<UserEntity[]>>;
  create: (name: string, description?: string) => Promise<ScyllaResult<OrganizationEntity>>;
  update: (
    organizationId: string,
    name?: string,
    description?: string,
  ) => Promise<ScyllaResult<OrganizationEntity>>;
  delete: (organizationId: string) => Promise<ScyllaResult<void>>;
}
