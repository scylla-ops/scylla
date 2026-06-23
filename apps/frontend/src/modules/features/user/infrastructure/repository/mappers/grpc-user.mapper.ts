import type { ListUsersResponse, UserResponse } from '@/generated/user.ts';

import type { PaginationInfo } from '@shared/domain/structs/pagination.struct.ts';
import type { UserEntity } from '@/modules/features/user/domain/entities/user.entity.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import { idValue, timestampToIso } from '@shared/infrastructure/grpc/wrappers.ts';

export class GrpcUserMapper {
  static toDomain(user: UserResponse): UserEntity {
    return {
      username: user.username,
      userId: idValue(user.userId),
      createdAt: timestampToIso(user.createdAt),
    };
  }

  static toDomainList(list: ListUsersResponse): PaginatedList<UserEntity> {
    return {
      items: list.users.map(GrpcUserMapper.toDomain),
      pagination: list.pagination as PaginationInfo,
    };
  }
}
