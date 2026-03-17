import type { ListOrganizationsResponse } from '@/generated/organization.ts';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';

export interface OrganizationRepository {
  getAll(): Promise<ScyllaResult<ListOrganizationsResponse>>;
  create: (name: string) => Promise<ScyllaResult<void>>;
}
