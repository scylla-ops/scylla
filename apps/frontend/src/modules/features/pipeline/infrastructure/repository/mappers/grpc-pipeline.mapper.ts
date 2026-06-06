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
import { Shell } from '@/generated/common.ts';
import type { PaginationInfo } from '@shared/domain/models/pagination.model.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import { idValue, timestampToIso, wrapId } from '@core/infrastructure/grpc/wrappers.ts';

function shellFromProto(s: Shell): 'sh' | 'bash' {
  return s === Shell.BASH ? 'bash' : 'sh';
}

function shellToProto(s: 'sh' | 'bash'): Shell {
  return s === 'bash' ? Shell.BASH : Shell.SH;
}

export class GrpcPipelineMapper {
  private static nodeToDomain(node: PipelineNode): PipelineStep {
    const base = {
      id: idValue(node.nodeId),
      deps: node.deps.map(idValue),
      ...(node.workingDir ? { workingDir: node.workingDir } : {}),
      env: Object.fromEntries(
        node.env.map(e => [e.key, e.source.oneofKind === 'value' ? e.source.value : '']),
      ),
    };

    const step = node.step;
    if (step.oneofKind === 'script') {
      return {
        ...base,
        kind: 'script',
        script: step.script.script,
        shell: shellFromProto(step.script.shell),
      };
    }
    if (step.oneofKind === 'exec') {
      return {
        ...base,
        kind: 'exec',
        command: step.exec.command,
        args: step.exec.args,
      };
    }
    return { ...base, kind: 'exec', command: '', args: [] };
  }

  static nodeFromDomain(step: PipelineStep): PipelineNode {
    return {
      nodeId: wrapId(step.id),
      deps: step.deps.map(wrapId),
      workingDir: step.workingDir ?? '',
      env: Object.entries(step.env).map(([key, value]) => ({
        key,
        source: { oneofKind: 'value', value },
      })),
      step:
        step.kind === 'script'
          ? {
              oneofKind: 'script',
              script: { script: step.script, shell: shellToProto(step.shell) },
            }
          : {
              oneofKind: 'exec',
              exec: { command: step.command, args: step.args },
            },
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
