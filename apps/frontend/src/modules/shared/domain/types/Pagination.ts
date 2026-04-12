export interface PaginationParams {
  page: number;
  pageSize: number;
}

export interface PaginationInfo {
  totalCount: number;
  page: number;
  pageSize: number;
  totalPages: number;
  hasNext: boolean;
  hasPrevious: boolean;
}

export const DEFAULT_PAGE_SIZE = 10;
