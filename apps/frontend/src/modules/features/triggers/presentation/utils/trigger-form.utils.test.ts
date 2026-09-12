import { describe, it, expect } from 'vitest';
import {
  convertCronToUTC,
  convertCronToLocal,
  cronFieldCount,
  isCronExpressionValid,
  triggerToDraftInputs,
  buildTriggerDraft,
} from './trigger-form.utils';
import { TriggerKind } from '@/modules/features/triggers/domain/structs/trigger-source.struct.ts';
import type { TriggerEntity } from '@/modules/features/triggers/domain/entities/trigger.entity.ts';

describe('cronFieldCount', () => {
  it('counts whitespace-separated fields', () => {
    expect(cronFieldCount('30 9 * * *')).toBe(5);
    expect(cronFieldCount('  30   9  *  *  * ')).toBe(5);
  });

  it('is 0 for an empty (or all-whitespace) expression', () => {
    expect(cronFieldCount('')).toBe(0);
    expect(cronFieldCount('   ')).toBe(0);
  });
});

describe('isCronExpressionValid', () => {
  it('requires exactly 5 fields', () => {
    expect(isCronExpressionValid('30 9 * * *')).toBe(true);
    expect(isCronExpressionValid('30 9 * *')).toBe(false);
    expect(isCronExpressionValid('30 9 * * * *')).toBe(false);
    expect(isCronExpressionValid('')).toBe(false);
  });
});

describe('convertCronToUTC / convertCronToLocal', () => {
  it('leaves a malformed (non-5-field) expression untouched', () => {
    expect(convertCronToUTC('not a cron')).toBe('not a cron');
    expect(convertCronToLocal('30 9 * *')).toBe('30 9 * *');
  });

  it('leaves an expression with a non-numeric minute or hour untouched (nothing to shift)', () => {
    expect(convertCronToUTC('*/15 * * * *')).toBe('*/15 * * * *');
    expect(convertCronToUTC('30 * * * *')).toBe('30 * * * *');
  });

  // The exact hour/day shift depends on the host's local timezone offset, which
  // this suite must not hardcode (it varies by machine and by DST). What must
  // hold everywhere is the round trip: converting to UTC and back to local
  // (or the reverse) with the day fields left wide open ('*') returns the
  // original expression exactly, because both legs read the same "now" and a
  // shift composed with its own inverse is the identity.
  it.each(['30 9 * * *', '0 0 * * *', '15 23 * * *', '45 6 * * 1-5'])(
    'round-trips %s through UTC and back to local',
    expression => {
      expect(convertCronToLocal(convertCronToUTC(expression))).toBe(expression);
    },
  );

  it.each(['30 9 * * *', '0 0 * * *', '15 23 * * *'])(
    'round-trips %s through local and back to UTC',
    expression => {
      expect(convertCronToUTC(convertCronToLocal(expression))).toBe(expression);
    },
  );
});

describe('triggerToDraftInputs', () => {
  it('flattens each input to its key/kind/value', () => {
    const trigger = {
      inputs: [
        { key: 'branch', value: { kind: 'literal', value: 'main' } },
        { key: 'sha', value: { kind: 'jsonPointer', value: '/head/sha' } },
      ],
    } as unknown as TriggerEntity;

    expect(triggerToDraftInputs(trigger)).toEqual([
      { key: 'branch', valueKind: 'literal', value: 'main' },
      { key: 'sha', valueKind: 'jsonPointer', value: '/head/sha' },
    ]);
  });

  it('returns an empty list for an undefined trigger', () => {
    expect(triggerToDraftInputs(undefined)).toEqual([]);
  });
});

describe('buildTriggerDraft', () => {
  it('trims the name and drops inputs with a blank key', () => {
    const draft = buildTriggerDraft({
      name: '  github-push  ',
      kind: TriggerKind.Webhook,
      cronExpression: '',
      signatureHeader: '  X-Hub-Signature  ',
      inputs: [
        { key: '  branch  ', valueKind: 'literal', value: 'main' },
        { key: '   ', valueKind: 'literal', value: 'dropped' },
      ],
    });

    expect(draft.name).toBe('github-push');
    expect(draft.inputs).toEqual([{ key: 'branch', value: { kind: 'literal', value: 'main' } }]);
    expect(draft.source).toEqual({ kind: TriggerKind.Webhook, signatureHeader: 'X-Hub-Signature' });
  });

  it('a jsonPointer input kind is only honored for a Webhook trigger', () => {
    const webhookDraft = buildTriggerDraft({
      name: 'x',
      kind: TriggerKind.Webhook,
      cronExpression: '',
      signatureHeader: '',
      inputs: [{ key: 'sha', valueKind: 'jsonPointer', value: '/head/sha' }],
    });
    expect(webhookDraft.inputs).toEqual([
      { key: 'sha', value: { kind: 'jsonPointer', value: '/head/sha' } },
    ]);

    const cronDraft = buildTriggerDraft({
      name: 'x',
      kind: TriggerKind.Cron,
      cronExpression: '0 0 * * *',
      signatureHeader: '',
      inputs: [{ key: 'sha', valueKind: 'jsonPointer', value: '/head/sha' }],
    });
    // A cron trigger has no webhook payload to point into - always literal.
    expect(cronDraft.inputs).toEqual([{ key: 'sha', value: { kind: 'literal', value: '/head/sha' } }]);
  });

  it('a Cron draft converts the expression (still a well-formed 5-field cron) and carries no signatureHeader', () => {
    const draft = buildTriggerDraft({
      name: 'nightly',
      kind: TriggerKind.Cron,
      cronExpression: ' 0 3 * * * ',
      signatureHeader: 'ignored-for-cron',
      inputs: [],
    });

    expect(draft.source.kind).toBe(TriggerKind.Cron);
    expect(draft.source).not.toHaveProperty('signatureHeader');
    if (draft.source.kind === TriggerKind.Cron) {
      expect(isCronExpressionValid(draft.source.expression)).toBe(true);
    }
  });
});
