import type { OrganizationRepository } from '@/modules/features/organization/domain/repository/organization.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { ListOrganizationsResponse } from '@/generated/scylla/organization/v1/organization.ts';

export default class GetUserOrganizationsUseCase {
  constructor(private readonly repository: OrganizationRepository) {}

  public async execute(): Promise<ScyllaResult<ListOrganizationsResponse>> {
    return this.repository.getMine();
  }
}
