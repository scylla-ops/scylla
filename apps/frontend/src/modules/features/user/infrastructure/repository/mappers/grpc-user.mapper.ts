import type { ListUsersResponse, UserResponse } from '@/generated/user.ts';

import type { PaginationInfo } from '@shared/domain/models/pagination.model.ts';
import type { User, UserList } from '@/modules/features/user/domain/models/user.model.ts';

export class GrpcUserMapper {
  static toDomain(user: UserResponse): User {
    return {
      username: user.username,
      userId: user.userId,
      createdAt: user.createdAt,
    };
  }

  static toDomainList(list: ListUsersResponse): UserList {
    return {
      users: list.users.map(GrpcUserMapper.toDomain),
      pagination: list.pagination as PaginationInfo,
    };
  }
}
