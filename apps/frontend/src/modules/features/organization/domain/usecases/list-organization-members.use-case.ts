import type { OrganizationRepository } from '@/modules/features/organization/domain/repository/organization.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { UserEntity } from '@/modules/features/user/domain/entities/user.entity.ts';

export default class ListOrganizationMembersUseCase {
  constructor(private readonly repository: OrganizationRepository) {}

  public async execute(organizationId: string): Promise<ScyllaResult<UserEntity[]>> {
    return this.repository.listMembers(organizationId);
  }
}
