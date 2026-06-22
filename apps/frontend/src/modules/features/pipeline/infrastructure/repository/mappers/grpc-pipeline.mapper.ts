import type {
  EnvEntry,
  Pipeline,
  PipelineMetadata,
  PipelineStep,
} from '@/modules/features/pipeline/domain/models/pipeline.model.ts';
import type {
  EnvVar,
  ListPipelinesResponse,
  PipelineNode,
  PipelineResponse,
  PipelineSummary,
} from '@/generated/pipeline.ts';
import { Shell } from '@/generated/common.ts';
import type { PaginationInfo } from '@shared/domain/models/pagination.model.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import { idValue, timestampToIso, wrapId } from '@shared/infrastructure/grpc/wrappers.ts';

function shellFromProto(s: Shell): 'sh' | 'bash' {
  return s === Shell.BASH ? 'bash' : 'sh';
}

function shellToProto(s: 'sh' | 'bash'): Shell {
  return s === 'bash' ? Shell.BASH : Shell.SH;
}

/** proto EnvVar -> domain EnvEntry (a secret ref or an inline literal). */
function envVarToDomain(e: EnvVar): EnvEntry {
  if (e.source.oneofKind === 'secretRef') {
    return { key: e.key, kind: 'secret', secretRef: e.source.secretRef };
  }
  return {
    key: e.key,
    kind: 'literal',
    value: e.source.oneofKind === 'value' ? e.source.value : '',
  };
}

/** domain EnvEntry -> proto EnvVar. */
function envVarFromDomain(entry: EnvEntry): EnvVar {
  if (entry.kind === 'secret') {
    return { key: entry.key, source: { oneofKind: 'secretRef', secretRef: entry.secretRef } };
  }
  return { key: entry.key, source: { oneofKind: 'value', value: entry.value } };
}

export class GrpcPipelineMapper {
  private static nodeToDomain(node: PipelineNode): PipelineStep {
    const base = {
      id: idValue(node.nodeId),
      deps: node.deps.map(idValue),
      ...(node.workingDir ? { workingDir: node.workingDir } : {}),
      env: node.env.map(envVarToDomain),
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
      env: step.env.map(envVarFromDomain),
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
