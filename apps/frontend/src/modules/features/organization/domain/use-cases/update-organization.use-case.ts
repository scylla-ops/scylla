import type { OrganizationRepository } from '@/modules/features/organization/domain/repository/organization.repository.ts';

export default class UpdateOrganizationUseCase {
  constructor(private readonly _repository: OrganizationRepository) {}

  public execute = (organizationId: string, name?: string, description?: string) =>
    this._repository.update(organizationId, name, description);
}

