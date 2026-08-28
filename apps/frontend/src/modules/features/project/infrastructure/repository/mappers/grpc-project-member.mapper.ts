import type { ProjectMember as GrpcProjectMember } from '@/generated/scylla/project/v1/project.ts';
import type { ProjectMember } from '@/modules/features/project/domain/structs/project-member.struct.ts';
import { idValue } from '@shared/infrastructure/grpc/wrappers.ts';

export class GrpcProjectMemberMapper {
  public static toDomain(member: GrpcProjectMember): ProjectMember {
    return {
      userId: idValue(member.userId),
      username: member.username,
    };
  }
}
