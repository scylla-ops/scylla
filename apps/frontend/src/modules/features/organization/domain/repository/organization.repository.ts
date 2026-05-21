import type { ListOrganizationsResponse, OrganizationResponse } from '@/generated/organization.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

export interface OrganizationRepository {
  getAll(): Promise<ScyllaResult<ListOrganizationsResponse>>;
  getMine(): Promise<ScyllaResult<ListOrganizationsResponse>>;
  create: (name: string, description?: string) => Promise<ScyllaResult<OrganizationResponse>>;
  update: (
    organizationId: string,
    name?: string,
    description?: string,
  ) => Promise<ScyllaResult<OrganizationResponse>>;
  delete: (organizationId: string) => Promise<ScyllaResult<void>>;
}
