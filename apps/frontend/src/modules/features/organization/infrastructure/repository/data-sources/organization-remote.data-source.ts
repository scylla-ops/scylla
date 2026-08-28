import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  ListOrganizationsResponse,
  Organization,
  OrganizationMember,
} from '@/generated/scylla/organization/v1/organization.ts';

export interface OrganizationRemoteDataSource {
  getAll: () => Promise<ScyllaResult<ListOrganizationsResponse>>;
  listMembers: (organizationId: string) => Promise<ScyllaResult<OrganizationMember[]>>;
  getMine: () => Promise<ScyllaResult<ListOrganizationsResponse>>;
  create: (name: string, description?: string) => Promise<ScyllaResult<Organization>>;
  update: (
    organizationId: string,
    name?: string,
    description?: string,
  ) => Promise<ScyllaResult<Organization>>;
  delete: (organizationId: string) => Promise<ScyllaResult<void>>;
}
