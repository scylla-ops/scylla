import type { OrganizationRepository } from '@/modules/features/organization/domain/repository/organization.repository.ts';
import type { OrganizationMember } from '@/modules/features/organization/domain/structs/organization-member.struct.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

export default class ListOrganizationMembersUseCase {
  constructor(private readonly repository: OrganizationRepository) {}

  public async execute(organizationId: string): Promise<ScyllaResult<OrganizationMember[]>> {
    return this.repository.listMembers(organizationId);
  }
}
