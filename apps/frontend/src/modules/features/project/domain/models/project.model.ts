import type { PaginationInfo } from '@shared/domain/models/pagination.model.ts';

export interface Project {
  id: string;
  name: string;
  description: string;
}

export interface ProjectList {
  projects: Project[];
  pagination: PaginationInfo;
}
