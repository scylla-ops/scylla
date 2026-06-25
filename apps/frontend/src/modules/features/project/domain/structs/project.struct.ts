import type { PaginationInfo } from '@shared/domain/structs/pagination.struct.ts';
import type { ProjectEntity } from '@/modules/features/project/domain/entities/project.entity.ts';

export interface ProjectList {
  projects: ProjectEntity[];
  pagination: PaginationInfo;
}
