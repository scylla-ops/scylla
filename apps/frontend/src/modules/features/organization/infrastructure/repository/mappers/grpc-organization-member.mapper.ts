import type { OrganizationMember } from '@/generated/scylla/organization/v1/organization.ts';
import type { OrganizationMemberEntity } from '@/modules/features/organization/domain/entities/organization-member.entity.ts';
import { idValue } from '@shared/infrastructure/grpc/wrappers.ts';

/** Maps proto `OrganizationMember` messages to the domain entity. */
export const GrpcOrganizationMemberMapper = {
  toDomain(member: OrganizationMember): OrganizationMemberEntity {
    return {
      userId: idValue(member.userId),
      username: member.username,
    };
  },
};
