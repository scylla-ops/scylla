import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import type { UserEntity } from '@/modules/features/user/domain/entities/user.entity.ts';

/** A paginated list of users (matches the gRPC ListUsers response shape). */
export type UserList = PaginatedList<UserEntity>;
