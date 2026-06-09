import type { ListUsersResponse, UserResponse } from '@/generated/user.ts';

import type { PaginationInfo } from '@shared/domain/models/pagination.model.ts';
import type { User } from '@/modules/features/user/domain/models/user.model.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import { idValue, timestampToIso } from '@core/infrastructure/grpc/wrappers.ts';

export class GrpcUserMapper {
  static toDomain(user: UserResponse): User {
    return {
      username: user.username,
      userId: idValue(user.userId),
      createdAt: timestampToIso(user.createdAt),
    };
  }

  static toDomainList(list: ListUsersResponse): PaginatedList<User> {
    return {
      items: list.users.map(GrpcUserMapper.toDomain),
      pagination: list.pagination as PaginationInfo,
    };
  }
}
