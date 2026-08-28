import type {
  ListOrganizationsResponse,
  Organization,
} from '@/generated/scylla/organization/v1/organization.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { UserEntity } from '@/modules/features/user/domain/entities/user.entity.ts';

export interface OrganizationRepository {
  getAll(): Promise<ScyllaResult<ListOrganizationsResponse>>;
  getMine(): Promise<ScyllaResult<ListOrganizationsResponse>>;
  listMembers(organizationId: string): Promise<ScyllaResult<UserEntity[]>>;
  create: (name: string, description?: string) => Promise<ScyllaResult<Organization>>;
  update: (
    organizationId: string,
    name?: string,
    description?: string,
  ) => Promise<ScyllaResult<Organization>>;
  delete: (organizationId: string) => Promise<ScyllaResult<void>>;
}
