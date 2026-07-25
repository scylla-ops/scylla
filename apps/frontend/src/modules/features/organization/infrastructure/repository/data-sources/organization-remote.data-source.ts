import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  ListOrganizationsResponse,
  Organization,
} from '@/generated/scylla/organization/v1/organization.ts';
import type { OrganizationMemberEntity } from '@/modules/features/organization/domain/entities/organization-member.entity.ts';

export interface OrganizationRemoteDataSource {
  getAll: () => Promise<ScyllaResult<ListOrganizationsResponse>>;
  getMine: () => Promise<ScyllaResult<ListOrganizationsResponse>>;
  create: (name: string, description?: string) => Promise<ScyllaResult<Organization>>;
  update: (
    organizationId: string,
    name?: string,
    description?: string,
  ) => Promise<ScyllaResult<Organization>>;
  delete: (organizationId: string) => Promise<ScyllaResult<void>>;
  listMembers: (organizationId: string) => Promise<ScyllaResult<OrganizationMemberEntity[]>>;
  addMember: (organizationId: string, userId: string) => Promise<ScyllaResult<void>>;
  removeMember: (organizationId: string, userId: string) => Promise<ScyllaResult<void>>;
}
