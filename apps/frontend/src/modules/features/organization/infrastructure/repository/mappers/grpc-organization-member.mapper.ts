import type { OrganizationMember as GrpcOrganizationMember } from '@/generated/scylla/organization/v1/organization.ts';
import type { OrganizationMember } from '@/modules/features/organization/domain/structs/organization-member.struct.ts';
import { idValue } from '@shared/infrastructure/grpc/wrappers.ts';

export class GrpcOrganizationMemberMapper {
  public static toDomain(member: GrpcOrganizationMember): OrganizationMember {
    return {
      userId: idValue(member.userId),
      username: member.username,
    };
  }
}
