import type { ListOrganizationsResponse, OrganizationResponse } from '@/generated/organization.ts';
import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';

export interface OrganizationRepository {
  getAll(): Promise<ScyllaResult<ListOrganizationsResponse>>;
  create: (name: string) => Promise<ScyllaResult<OrganizationResponse>>;
}
