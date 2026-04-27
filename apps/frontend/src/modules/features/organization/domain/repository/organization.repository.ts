import type { ListOrganizationsResponse, OrganizationResponse } from '@/generated/organization.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

export interface OrganizationRepository {
  getAll(): Promise<ScyllaResult<ListOrganizationsResponse>>;
  create: (name: string) => Promise<ScyllaResult<OrganizationResponse>>;
}
