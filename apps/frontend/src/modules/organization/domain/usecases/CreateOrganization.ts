import type { OrganizationRepository } from '@/modules/organization/domain/repository/OrganizationRepository.ts';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';

export default class CreateOrganization {
  constructor(private readonly organizationRepository: OrganizationRepository) {}

  public execute(name: string): Promise<ScyllaResult<void>> {
    return this.organizationRepository.create(name);
  }
}
