import type { PaginationInfo } from '@shared/domain/structs/pagination.struct.ts';

export type PaginatedList<T> = {
  readonly items: T[];
  readonly pagination: PaginationInfo;
};
