import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { ListOrganizationsResponse, OrganizationResponse } from '@/generated/organization.ts';

export interface OrganizationRemoteDataSource {
  getAll: () => Promise<ScyllaResult<ListOrganizationsResponse>>;
  create: (name: string) => Promise<ScyllaResult<OrganizationResponse>>;
}
