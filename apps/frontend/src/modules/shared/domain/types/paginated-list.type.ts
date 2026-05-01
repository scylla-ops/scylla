import type { PaginationInfo } from '@shared/domain/models/pagination.model.ts';

export type PaginatedList<T> = {
  readonly items: T[];
  readonly pagination: PaginationInfo;
};
