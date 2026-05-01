import type { OrganizationRepository } from '@/modules/features/organization/domain/repository/organization.repository.ts';

export default class DeleteOrganizationUseCase {
  constructor(private readonly _repository: OrganizationRepository) {}

  public execute = (organizationId: string) => this._repository.delete(organizationId);
}

