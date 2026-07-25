import type { OrganizationRepository } from '@/modules/features/organization/domain/repository/organization.repository.ts';

export default class RemoveOrganizationMemberUseCase {
  constructor(private readonly _repository: OrganizationRepository) {}

  public execute = (organizationId: string, userId: string) =>
    this._repository.removeMember(organizationId, userId);
}
