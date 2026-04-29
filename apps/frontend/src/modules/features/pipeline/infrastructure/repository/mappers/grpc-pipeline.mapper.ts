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

export class GrpcPipelineMapper {
  private static nodeToDomain(node: PipelineNode): PipelineStep {
    return {
      id: node.nodeId,
      deps: node.deps,
      command: node.command,
      args: node.args,
    };
  }

  static toDomain(pipeline: PipelineResponse): Pipeline {
    return {
      info: {
        id: pipeline.pipelineId,
        projectId: pipeline.projectId,
        name: pipeline.name,
        nodeCount: pipeline.nodes.length,
        createdAt: pipeline.createdAt,
        updatedAt: pipeline.updatedAt,
      },
      steps: pipeline.nodes.map(GrpcPipelineMapper.nodeToDomain),
    };
  }

  private static toDomainInfo(pipeline: PipelineSummary): PipelineMetadata {
    return {
      id: pipeline.pipelineId,
      projectId: pipeline.projectId,
      name: pipeline.name,
      nodeCount: pipeline.nodeCount,
      createdAt: pipeline.createdAt,
      updatedAt: pipeline.updatedAt,
    };
  }

  static toDomainInfoList(pipelines: ListPipelinesResponse): PaginatedList<PipelineMetadata> {
    return {
      items: pipelines.pipelines.map(GrpcPipelineMapper.toDomainInfo),
      pagination: pipelines.pagination as PaginationInfo,
    };
  }
}
