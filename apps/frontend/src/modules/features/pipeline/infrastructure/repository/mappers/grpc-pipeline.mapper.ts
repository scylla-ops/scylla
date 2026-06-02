import type {
  Pipeline,
  PipelineMetadata,
  PipelineStep,
} from '@/modules/features/pipeline/domain/models/pipeline.model.ts';
import type {
  ListPipelinesResponse,
  PipelineNode,
  PipelineResponse,
  PipelineSummary,
} from '@/generated/pipeline.ts';
import type { PaginationInfo } from '@shared/domain/models/pagination.model.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import { idValue, timestampToIso, wrapId } from '@core/infrastructure/grpc/wrappers.ts';

export class GrpcPipelineMapper {
  private static nodeToDomain(node: PipelineNode): PipelineStep {
    return {
      id: idValue(node.nodeId),
      deps: node.deps.map(idValue),
      command: node.command,
      args: node.args,
    };
  }

  static nodeFromDomain(step: PipelineStep): PipelineNode {
    return {
      nodeId: wrapId(step.id),
      deps: step.deps.map(wrapId),
      command: step.command,
      args: step.args,
    };
  }

  static toDomain(pipeline: PipelineResponse): Pipeline {
    return {
      id: idValue(pipeline.pipelineId),
      projectId: idValue(pipeline.projectId),
      name: pipeline.name,
      nodes: pipeline.nodes.map(GrpcPipelineMapper.nodeToDomain),
    };
  }

  private static toDomainInfo(pipeline: PipelineSummary): PipelineMetadata {
    return {
      id: idValue(pipeline.pipelineId),
      projectId: idValue(pipeline.projectId),
      name: pipeline.name,
      nodeCount: pipeline.nodeCount,
      createdAt: timestampToIso(pipeline.createdAt),
      updatedAt: timestampToIso(pipeline.updatedAt),
    };
  }

  static toDomainInfoList(pipelines: ListPipelinesResponse): PaginatedList<PipelineMetadata> {
    return {
      items: pipelines.pipelines.map(GrpcPipelineMapper.toDomainInfo),
      pagination: pipelines.pagination as PaginationInfo,
    };
  }
}
