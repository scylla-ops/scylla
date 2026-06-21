import type {
  TriggerDraft,
  TriggerEntity,
} from '@/modules/features/triggers/domain/entities/trigger.entity.ts';
import { TriggerKind } from '@/modules/features/triggers/domain/models/trigger-source.model.ts';

/**
 * Convert a Cron 5-field expression from the local timezone to UTC.
 */
export function convertCronToUTC(cronExpression: string): string {
  const fields = cronExpression.trim().split(/\s+/);
  if (fields.length !== 5) return cronExpression;

  const [minute, hour, dayOfMonth, month, dayOfWeek] = fields;

  if (isNaN(Number(minute)) || isNaN(Number(hour))) {
    return cronExpression;
  }

  const now = new Date();
  const localDate = new Date(
    now.getFullYear(),
    now.getMonth(),
    now.getDate(),
    Number(hour),
    Number(minute),
  );

  const utcMinute = String(localDate.getUTCMinutes());
  const utcHour = String(localDate.getUTCHours());

  return `${utcMinute} ${utcHour} ${dayOfMonth} ${month} ${dayOfWeek}`;
}

/**
 * Convert a Cron 5-field expression from UTC to the local timezone.
 */
export function convertCronToLocal(cronExpression: string): string {
  const fields = cronExpression.trim().split(/\s+/);
  if (fields.length !== 5) return cronExpression;

  const [minute, hour, dayOfMonth, month, dayOfWeek] = fields;

  if (isNaN(Number(minute)) || isNaN(Number(hour))) {
    return cronExpression;
  }

  // On crée une date fictive en UTC
  const now = new Date();
  const utcDate = new Date(
    Date.UTC(
      now.getUTCFullYear(),
      now.getUTCMonth(),
      now.getUTCDate(),
      Number(hour),
      Number(minute),
    ),
  );

  // Récupération automatique à l'heure de l'ordinateur
  const localMinute = String(utcDate.getMinutes());
  const localHour = String(utcDate.getHours());

  return `${localMinute} ${localHour} ${dayOfMonth} ${month} ${dayOfWeek}`;
}

/** A trigger input as edited in the form (flat, string-only). */
export interface DraftInput {
  key: string;
  valueKind: 'literal' | 'jsonPointer';
  value: string;
}

/** Number of whitespace-separated fields in a cron expression. */
export const cronFieldCount = (expression: string): number => {
  const trimmed = expression.trim();
  return trimmed.length === 0 ? 0 : trimmed.split(/\s+/).length;
};

/** A 5-field cron expression (min hour day month weekday). Server owns the rest. */
export const isCronExpressionValid = (expression: string): boolean =>
  cronFieldCount(expression) === 5;

export const triggerToDraftInputs = (trigger?: TriggerEntity): DraftInput[] =>
  (trigger?.inputs ?? []).map(input => ({
    key: input.key,
    valueKind: input.value.kind,
    value: input.value.value,
  }));

export const buildTriggerDraft = (params: {
  name: string;
  kind: TriggerKind;
  cronExpression: string;
  signatureHeader: string;
  inputs: DraftInput[];
}): TriggerDraft => {
  const allowPointer = params.kind === TriggerKind.Webhook;

  const inputs = params.inputs
    .filter(input => input.key.trim().length > 0)
    .map(input =>
      allowPointer && input.valueKind === 'jsonPointer'
        ? { key: input.key.trim(), value: { kind: 'jsonPointer', value: input.value } as const }
        : { key: input.key.trim(), value: { kind: 'literal', value: input.value } as const },
    );

  let finalCronExpression = params.cronExpression.trim();

  if (params.kind === TriggerKind.Cron) {
    finalCronExpression = convertCronToUTC(finalCronExpression);
  }

  return {
    name: params.name.trim(),
    source:
      params.kind === TriggerKind.Cron
        ? { kind: TriggerKind.Cron, expression: finalCronExpression }
        : { kind: TriggerKind.Webhook, signatureHeader: params.signatureHeader.trim() },
    inputs,
  };
};
