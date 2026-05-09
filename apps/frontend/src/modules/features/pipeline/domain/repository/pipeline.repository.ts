import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PaginationParams } from '@shared/domain/models/pagination.model.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import type {
  Pipeline,
  PipelineMetadata,
  PipelineStep,
} from '@/modules/features/pipeline/domain/models/pipeline.model.ts';

export interface PipelineRepository {
  getMetadataByProjectId(
    projectId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<PaginatedList<PipelineMetadata>>>;
  deleteById(id: string): Promise<ScyllaResult<void>>;
  run(id: string): Promise<ScyllaResult<void>>;
  create(pipeline: Omit<Pipeline, 'id'>): Promise<ScyllaResult<void>>;
  getById(id: string): Promise<ScyllaResult<Pipeline>>;
  edit(id: string, steps: PipelineStep[], name?: string): Promise<ScyllaResult<Pipeline>>;
}
