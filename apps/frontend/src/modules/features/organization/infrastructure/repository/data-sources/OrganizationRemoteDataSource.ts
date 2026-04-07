import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { ListOrganizationsResponse } from '@/generated/organization.ts';

export interface OrganizationRemoteDataSource {
  getAll: () => Promise<ScyllaResult<ListOrganizationsResponse>>;
  create: (name: string) => Promise<ScyllaResult<void>>;
}
