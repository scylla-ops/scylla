import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PaginationParams } from '@shared/domain/structs/pagination.struct.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import type { PipelineEntity } from '@/modules/features/pipeline/domain/entities/pipeline.entity.ts';
import type {
  PipelineMetadata,
  PipelineStep,
} from '@/modules/features/pipeline/domain/structs/pipeline.struct.ts';

export interface PipelineRepository {
  getMetadataByProjectId(
    projectId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<PaginatedList<PipelineMetadata>>>;
  deleteById(id: string): Promise<ScyllaResult<void>>;
  run(id: string): Promise<ScyllaResult<void>>;
  create(pipeline: Omit<PipelineEntity, 'id'>): Promise<ScyllaResult<void>>;
  getById(id: string): Promise<ScyllaResult<PipelineEntity>>;
  edit(id: string, steps: PipelineStep[], name?: string): Promise<ScyllaResult<PipelineEntity>>;
}
