import type { ListPipelinesResponse } from '@/generated/pipeline.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PaginationParams } from '@/modules/shared/domain/models/pagination.model.ts';

export interface PipelineRemoteDataSource {
  getByProjectId(
    projectId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<ListPipelinesResponse>>;
  deleteById(id: string): Promise<ScyllaResult<void>>;
  run(id: string): Promise<ScyllaResult<void>>;
  create: (content: string) => Promise<ScyllaResult<void>>;
}
