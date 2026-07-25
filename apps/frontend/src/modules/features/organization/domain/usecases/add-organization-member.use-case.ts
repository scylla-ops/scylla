import type { OrganizationRepository } from '@/modules/features/organization/domain/repository/organization.repository.ts';

export default class AddOrganizationMemberUseCase {
  constructor(private readonly _repository: OrganizationRepository) {}

  public execute = (organizationId: string, userId: string) =>
    this._repository.addMember(organizationId, userId);
}
