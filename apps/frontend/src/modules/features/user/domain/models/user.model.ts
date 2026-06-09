import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';

export interface User {
  userId: string;
  username: string;
  createdAt: string;
}

/** A paginated list of users (matches the gRPC ListUsers response shape). */
export type UserList = PaginatedList<User>;
