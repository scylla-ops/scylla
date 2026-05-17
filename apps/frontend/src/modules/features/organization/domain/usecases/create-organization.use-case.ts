import type { OrganizationRepository } from '@/modules/features/organization/domain/repository/organization.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { OrganizationResponse } from '@/generated/organization.ts';

export default class CreateOrganizationUseCase {
  constructor(private readonly organizationRepository: OrganizationRepository) {}

  public execute(name: string, description?: string): Promise<ScyllaResult<OrganizationResponse>> {
    return this.organizationRepository.create(name, description);
  }
}
