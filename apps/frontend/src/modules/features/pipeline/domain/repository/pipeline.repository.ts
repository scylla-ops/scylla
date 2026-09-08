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
  /**
   * Pipeline metadata across a whole organization, in one call.
   *
   * The organization-wide views used to fan out one `getMetadataByProjectId`
   * per project — N requests, N cache entries, and one error toast per project
   * the caller could not read.
   */
  getMetadataByOrganizationId(
    organizationId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<PaginatedList<PipelineMetadata>>>;
  deleteById(id: string): Promise<ScyllaResult<void>>;
  run(id: string): Promise<ScyllaResult<void>>;
  create(pipeline: Omit<PipelineEntity, 'id'>): Promise<ScyllaResult<void>>;
  getById(id: string): Promise<ScyllaResult<PipelineEntity>>;
  edit(id: string, steps: PipelineStep[], name?: string): Promise<ScyllaResult<PipelineEntity>>;
}
