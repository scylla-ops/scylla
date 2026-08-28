import type { OrganizationMember as GrpcOrganizationMember } from '@/generated/scylla/organization/v1/organization.ts';
import { idValue } from '@shared/infrastructure/grpc/wrappers.ts';
import type { UserEntity } from '@/modules/features/user/domain/entities/user.entity.ts';

export class GrpcOrganizationMemberMapper {
  public static toDomain(member: GrpcOrganizationMember): UserEntity {
    return {
      userId: idValue(member.userId),
      username: member.username,
    };
  }
}
