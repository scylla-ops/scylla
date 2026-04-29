import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PaginationParams } from '@shared/domain/models/pagination.model.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import type {
  Pipeline,
  PipelineMetadata,
} from '@/modules/features/pipeline/domain/models/pipeline.model.ts';

export interface PipelineRepository {
  getMetadataByProjectId(
    projectId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<PaginatedList<PipelineMetadata>>>;
  deleteById(id: string): Promise<ScyllaResult<void>>;
  run(id: string): Promise<ScyllaResult<void>>;
  create: (content: string) => Promise<ScyllaResult<void>>;
  getById(id: string): Promise<ScyllaResult<Pipeline>>;
}
