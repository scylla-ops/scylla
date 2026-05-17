import type {
  CreatePipelineRequest,
  ListPipelinesResponse,
  PipelineNode,
  PipelineResponse,
} from '@/generated/pipeline.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { PaginationParams } from '@/modules/shared/domain/models/pagination.model.ts';

export interface PipelineRemoteDataSource {
  getByProjectId(
    projectId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<ListPipelinesResponse>>;
  deleteById(id: string): Promise<ScyllaResult<void>>;
  run(id: string): Promise<ScyllaResult<void>>;
  create(request: CreatePipelineRequest): Promise<ScyllaResult<void>>;
  getById(id: string): Promise<ScyllaResult<PipelineResponse>>;
  update(id: string, nodes: PipelineNode[], name?: string): Promise<ScyllaResult<PipelineResponse>>;
}
