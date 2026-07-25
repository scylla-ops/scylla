import type { OrganizationRepository } from '@/modules/features/organization/domain/repository/organization.repository.ts';

export default class ListOrganizationMembersUseCase {
  constructor(private readonly _repository: OrganizationRepository) {}

  public execute = (organizationId: string) => this._repository.listMembers(organizationId);
}
