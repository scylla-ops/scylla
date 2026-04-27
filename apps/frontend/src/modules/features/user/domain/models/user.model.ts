import type { PaginationInfo } from '@shared/domain/models/pagination.model.ts';

export interface UserList {
  users: User[];
  pagination: PaginationInfo;
}

export interface User {
  userId: string;
  username: string;
  createdAt: string;
}
