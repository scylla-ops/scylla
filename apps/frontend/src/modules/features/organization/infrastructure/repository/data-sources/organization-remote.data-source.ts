import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { ListOrganizationsResponse, OrganizationResponse } from '@/generated/organization.ts';

export interface OrganizationRemoteDataSource {
  getAll: () => Promise<ScyllaResult<ListOrganizationsResponse>>;
  create: (name: string) => Promise<ScyllaResult<OrganizationResponse>>;
  update: (organizationId: string, name?: string, description?: string) => Promise<ScyllaResult<OrganizationResponse>>;
  delete: (organizationId: string) => Promise<ScyllaResult<void>>;
}
