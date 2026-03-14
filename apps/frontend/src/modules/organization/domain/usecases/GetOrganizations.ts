import type { OrganizationRepository } from '@/modules/organization/domain/repository/OrganizationRepository.ts';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';
import type { ListOrganizationsResponse } from '@/generated/organization.ts';

export default class GetOrganizations {
  constructor(private readonly repository: OrganizationRepository) {}

  public async execute(): Promise<ScyllaResult<ListOrganizationsResponse>> {
    return this.repository.getAll();
  }
}
