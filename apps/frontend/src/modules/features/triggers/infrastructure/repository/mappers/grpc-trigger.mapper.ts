import type {
  CreatedTrigger as ProtoCreatedTrigger,
  CreateTriggerRequest,
  ListTriggersResponse,
  TriggerInput as ProtoTriggerInput,
  TriggerView,
  UpdateTriggerRequest,
} from '@/generated/trigger.ts';
import type {
  CreatedTrigger,
  TriggerDraft,
  TriggerEntity,
} from '@/modules/features/triggers/domain/entities/trigger.entity.ts';
import type {
  TriggerInput,
  TriggerSource,
} from '@/modules/features/triggers/domain/models/trigger-source.model.ts';
import { TriggerKind } from '@/modules/features/triggers/domain/models/trigger-source.model.ts';
import {
  idValue,
  timestampToIso,
  timestampToIsoOpt,
  wrapId,
} from '@shared/infrastructure/grpc/wrappers.ts';
import { ScyllaError } from '@shared/utils/scylla-result.ts';

/** Maps gRPC trigger messages ↔ the domain trigger entity. */
export class GrpcTriggerMapper {
  // ── proto → domain ─────────────────────────────────────────────────────────

  private static sourceToDomain(view: TriggerView): TriggerSource {
    switch (view.source.oneofKind) {
      case 'cron':
        return { kind: TriggerKind.Cron, expression: view.source.cron.expression };
      case 'webhook':
        return {
          kind: TriggerKind.Webhook,
          signatureHeader: view.source.webhook.signatureHeader,
          webhookUrl: view.webhookUrl,
        };
      default:
        // Defensive: unknown/empty source — render as an empty cron so the row still shows.
        return { kind: TriggerKind.Cron, expression: '' };
    }
  }

  private static inputToDomain(input: ProtoTriggerInput): TriggerInput {
    switch (input.source.oneofKind) {
      case 'literal':
        return { key: input.key, value: { kind: 'literal', value: input.source.literal } };
      case 'jsonPointer':
        return { key: input.key, value: { kind: 'jsonPointer', value: input.source.jsonPointer } };
      default:
        return { key: input.key, value: { kind: 'literal', value: '' } };
    }
  }

  static toDomain(view: TriggerView): TriggerEntity {
    return {
      id: idValue(view.triggerId),
      pipelineId: idValue(view.pipelineId),
      name: view.name,
      source: GrpcTriggerMapper.sourceToDomain(view),
      inputs: view.inputs.map(GrpcTriggerMapper.inputToDomain),
      enabled: view.enabled,
      nextFireAt: timestampToIsoOpt(view.nextFireAt),
      lastFiredAt: timestampToIsoOpt(view.lastFiredAt),
      lastStatus: view.lastStatus,
      createdAt: timestampToIso(view.createdAt),
      updatedAt: timestampToIso(view.updatedAt),
    };
  }

  static toDomainList(response: ListTriggersResponse): TriggerEntity[] {
    return response.triggers.map(GrpcTriggerMapper.toDomain);
  }

  static createdToDomain(response: ProtoCreatedTrigger): CreatedTrigger {
    if (!response.trigger) {
      throw new ScyllaError('CreateTrigger returned no trigger');
    }
    return {
      trigger: GrpcTriggerMapper.toDomain(response.trigger),
      webhookSecret: response.webhookSecret,
    };
  }

  // ── domain → proto ─────────────────────────────────────────────────────────

  private static inputToProto(input: TriggerInput): ProtoTriggerInput {
    return {
      key: input.key,
      source:
        input.value.kind === 'literal'
          ? { oneofKind: 'literal', literal: input.value.value }
          : { oneofKind: 'jsonPointer', jsonPointer: input.value.value },
    };
  }

  private static draftSourceToProto(draft: TriggerDraft): CreateTriggerRequest['source'] {
    return draft.source.kind === TriggerKind.Cron
      ? { oneofKind: 'cron', cron: { expression: draft.source.expression } }
      : { oneofKind: 'webhook', webhook: { signatureHeader: draft.source.signatureHeader } };
  }

  static draftToCreateRequest(pipelineId: string, draft: TriggerDraft): CreateTriggerRequest {
    return {
      pipelineId: wrapId(pipelineId),
      name: draft.name,
      source: GrpcTriggerMapper.draftSourceToProto(draft),
      inputs: draft.inputs.map(GrpcTriggerMapper.inputToProto),
    };
  }

  static draftToUpdateRequest(triggerId: string, draft: TriggerDraft): UpdateTriggerRequest {
    return {
      triggerId: wrapId(triggerId),
      name: draft.name,
      source: GrpcTriggerMapper.draftSourceToProto(draft),
      inputs: draft.inputs.map(GrpcTriggerMapper.inputToProto),
    };
  }
}
