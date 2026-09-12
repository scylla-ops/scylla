import { describe, it, expect } from 'vitest';
import { GrpcTriggerMapper } from './grpc-trigger.mapper';
import type { Trigger } from '@/generated/scylla/trigger/v1/trigger.ts';
import { TriggerKind } from '@/modules/features/triggers/domain/structs/trigger-source.struct.ts';
import type { TriggerDraft } from '@/modules/features/triggers/domain/entities/trigger.entity.ts';

const TS = { seconds: 1735689600n, nanos: 0 };
const ISO = '2025-01-01T00:00:00.000Z';

const baseTrigger = (overrides: Partial<Trigger> = {}): Trigger => ({
  triggerId: { value: 'trigger-1' },
  pipelineId: { value: 'pipeline-1' },
  name: 'github-push',
  inputs: [],
  createdAt: TS,
  updatedAt: TS,
  source: { oneofKind: 'webhook', webhook: { signatureHeader: '', url: 'https://x/webhooks/1' } },
  activation: { oneofKind: 'disabled', disabled: {} },
  ...overrides,
});

describe('GrpcTriggerMapper.toDomain', () => {
  it('unwraps ids and formats timestamps', () => {
    const domain = GrpcTriggerMapper.toDomain(baseTrigger());
    expect(domain.id).toBe('trigger-1');
    expect(domain.pipelineId).toBe('pipeline-1');
    expect(domain.createdAt).toBe(ISO);
    expect(domain.updatedAt).toBe(ISO);
  });

  describe('source', () => {
    it('maps a cron source', () => {
      const domain = GrpcTriggerMapper.toDomain(
        baseTrigger({ source: { oneofKind: 'cron', cron: { expression: '0 9 * * *' } } }),
      );
      expect(domain.source).toEqual({ kind: TriggerKind.Cron, expression: '0 9 * * *' });
    });

    it('maps a webhook source, including the server-derived url', () => {
      const domain = GrpcTriggerMapper.toDomain(
        baseTrigger({
          source: {
            oneofKind: 'webhook',
            webhook: { signatureHeader: 'X-Hub-Signature-256', url: 'https://x/webhooks/1' },
          },
        }),
      );
      expect(domain.source).toEqual({
        kind: TriggerKind.Webhook,
        signatureHeader: 'X-Hub-Signature-256',
        webhookUrl: 'https://x/webhooks/1',
      });
    });

    it('a source arm newer than this build maps to Unknown rather than a guess', () => {
      const domain = GrpcTriggerMapper.toDomain(baseTrigger({ source: { oneofKind: undefined } }));
      expect(domain.source).toEqual({ kind: TriggerKind.Unknown });
    });
  });

  describe('activation', () => {
    it('a disabled trigger has enabled:false and no nextFireAt', () => {
      const domain = GrpcTriggerMapper.toDomain(
        baseTrigger({ activation: { oneofKind: 'disabled', disabled: {} } }),
      );
      expect(domain.enabled).toBe(false);
      expect(domain.nextFireAt).toBeUndefined();
    });

    it('an enabled cron trigger surfaces its nextFireAt', () => {
      const domain = GrpcTriggerMapper.toDomain(
        baseTrigger({ activation: { oneofKind: 'enabled', enabled: { nextFireAt: TS } } }),
      );
      expect(domain.enabled).toBe(true);
      expect(domain.nextFireAt).toBe(ISO);
    });

    it('an enabled trigger the scheduler has not anchored yet has enabled:true but no nextFireAt', () => {
      const domain = GrpcTriggerMapper.toDomain(
        baseTrigger({ activation: { oneofKind: 'enabled', enabled: {} } }),
      );
      expect(domain.enabled).toBe(true);
      expect(domain.nextFireAt).toBeUndefined();
    });
  });

  describe('lastObservation', () => {
    it('is absent before the first fire', () => {
      const domain = GrpcTriggerMapper.toDomain(baseTrigger({ lastObservation: undefined }));
      expect(domain.lastFiredAt).toBeUndefined();
      expect(domain.lastResult).toBeUndefined();
    });

    it('a succeeded fire carries its firedAt and a succeeded result', () => {
      const domain = GrpcTriggerMapper.toDomain(
        baseTrigger({ lastObservation: { firedAt: TS, result: { oneofKind: 'succeeded', succeeded: {} } } }),
      );
      expect(domain.lastFiredAt).toBe(ISO);
      expect(domain.lastResult).toEqual({ kind: 'succeeded' });
    });

    it('a failed fire carries its error message', () => {
      const domain = GrpcTriggerMapper.toDomain(
        baseTrigger({
          lastObservation: {
            firedAt: TS,
            result: { oneofKind: 'failed', failed: { error: 'connection refused' } },
          },
        }),
      );
      expect(domain.lastResult).toEqual({ kind: 'failed', error: 'connection refused' });
    });

    it('a result arm newer than this build maps to unknown (neither success nor failure)', () => {
      const domain = GrpcTriggerMapper.toDomain(
        baseTrigger({ lastObservation: { firedAt: TS, result: { oneofKind: undefined } } }),
      );
      expect(domain.lastResult).toEqual({ kind: 'unknown' });
    });
  });

  describe('inputs', () => {
    it('maps a literal input', () => {
      const domain = GrpcTriggerMapper.toDomain(
        baseTrigger({ inputs: [{ key: 'branch', source: { oneofKind: 'literal', literal: 'main' } }] }),
      );
      expect(domain.inputs).toEqual([{ key: 'branch', value: { kind: 'literal', value: 'main' } }]);
    });

    it('maps a jsonPointer input', () => {
      const domain = GrpcTriggerMapper.toDomain(
        baseTrigger({
          inputs: [{ key: 'sha', source: { oneofKind: 'jsonPointer', jsonPointer: '/head/sha' } }],
        }),
      );
      expect(domain.inputs).toEqual([{ key: 'sha', value: { kind: 'jsonPointer', value: '/head/sha' } }]);
    });

    it('an input source arm newer than this build maps to an empty literal', () => {
      const domain = GrpcTriggerMapper.toDomain(
        baseTrigger({ inputs: [{ key: 'x', source: { oneofKind: undefined } }] }),
      );
      expect(domain.inputs).toEqual([{ key: 'x', value: { kind: 'literal', value: '' } }]);
    });
  });
});

describe('GrpcTriggerMapper.toDomainList', () => {
  it('maps every trigger in order', () => {
    const list = GrpcTriggerMapper.toDomainList([
      baseTrigger({ triggerId: { value: 'a' } }),
      baseTrigger({ triggerId: { value: 'b' } }),
    ]);
    expect(list.map(t => t.id)).toEqual(['a', 'b']);
  });
});

describe('GrpcTriggerMapper.createdToDomain', () => {
  it('carries the one-time webhook secret alongside the mapped trigger', () => {
    const created = GrpcTriggerMapper.createdToDomain({
      trigger: baseTrigger(),
      webhookSecret: 'super-secret',
    });
    expect(created.trigger.id).toBe('trigger-1');
    expect(created.webhookSecret).toBe('super-secret');
  });

  it('throws if the response carries no trigger (a malformed response, never expected in practice)', () => {
    expect(() => GrpcTriggerMapper.createdToDomain({})).toThrow('CreateTrigger returned no trigger');
  });
});

describe('GrpcTriggerMapper draft -> proto', () => {
  const cronDraft: TriggerDraft = {
    name: 'nightly',
    source: { kind: TriggerKind.Cron, expression: '0 0 * * *' },
    inputs: [{ key: 'branch', value: { kind: 'literal', value: 'main' } }],
  };

  const webhookDraft: TriggerDraft = {
    name: 'github-push',
    source: { kind: TriggerKind.Webhook, signatureHeader: 'X-Hub-Signature-256' },
    inputs: [{ key: 'sha', value: { kind: 'jsonPointer', value: '/head/sha' } }],
  };

  it('builds a CreateTriggerRequest for a cron draft', () => {
    const request = GrpcTriggerMapper.draftToCreateRequest('pipeline-1', cronDraft);
    expect(request).toEqual({
      pipelineId: { value: 'pipeline-1' },
      name: 'nightly',
      source: { oneofKind: 'cron', cron: { expression: '0 0 * * *' } },
      inputs: [{ key: 'branch', source: { oneofKind: 'literal', literal: 'main' } }],
    });
  });

  it('builds a CreateTriggerRequest for a webhook draft, its jsonPointer input included', () => {
    const request = GrpcTriggerMapper.draftToCreateRequest('pipeline-1', webhookDraft);
    expect(request).toEqual({
      pipelineId: { value: 'pipeline-1' },
      name: 'github-push',
      source: { oneofKind: 'webhook', webhook: { signatureHeader: 'X-Hub-Signature-256' } },
      inputs: [{ key: 'sha', source: { oneofKind: 'jsonPointer', jsonPointer: '/head/sha' } }],
    });
  });

  it('builds an UpdateTriggerRequest the same way, scoped by triggerId instead of pipelineId', () => {
    const request = GrpcTriggerMapper.draftToUpdateRequest('trigger-1', cronDraft);
    expect(request).toEqual({
      triggerId: { value: 'trigger-1' },
      name: 'nightly',
      source: { oneofKind: 'cron', cron: { expression: '0 0 * * *' } },
      inputs: [{ key: 'branch', source: { oneofKind: 'literal', literal: 'main' } }],
    });
  });
});
