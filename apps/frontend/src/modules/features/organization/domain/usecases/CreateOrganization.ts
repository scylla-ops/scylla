import type { OrganizationRepository } from '@/modules/features/organization/domain/repository/OrganizationRepository.ts';
import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { OrganizationResponse } from '@/generated/organization.ts';

export default class CreateOrganization {
  constructor(private readonly organizationRepository: OrganizationRepository) {}

  public execute(name: string): Promise<ScyllaResult<OrganizationResponse>> {
    return this.organizationRepository.create(name);
  }
}
