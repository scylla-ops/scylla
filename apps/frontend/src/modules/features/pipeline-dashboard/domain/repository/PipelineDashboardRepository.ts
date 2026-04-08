import type { ListPipelinesResponse } from '@/generated/pipeline.ts';
import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { PaginationParams } from '@/modules/shared/domain/types/Pagination.ts';

export interface PipelineDashboardRepository {
  getByProjectId(
    projectId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<ListPipelinesResponse>>;
  deleteById(id: string): Promise<ScyllaResult<void>>;
}
