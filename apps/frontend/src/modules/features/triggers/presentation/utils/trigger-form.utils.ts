import type {
  TriggerDraft,
  TriggerEntity,
} from '@/modules/features/triggers/domain/entities/trigger.entity.ts';
import { TriggerKind } from '@/modules/features/triggers/domain/models/trigger-source.model.ts';

/**
 * Décale un champ Cron (Jour du Mois ou Jour de la Semaine) selon un diff (-1, 0, 1)
 * Gère les listes (1,2,3) et les plages (1-5).
 */
function shiftCronField(field: string, shift: number, min: number, max: number): string {
  if (field === '*' || shift === 0) return field;

  const rangeSize = max - min + 1;

  // Helper pour décaler de manière cyclique en restant dans les bornes [min, max]
  const shiftVal = (val: number) => {
    let newVal = (val - min + shift) % rangeSize;
    if (newVal < 0) newVal += rangeSize;
    return newVal + min;
  };

  const parts = field.split(',');
  const resultParts: string[] = [];

  for (const part of parts) {
    if (part.includes('-')) {
      // Si c'est une plage (ex: 1-5), on la développe pour éviter de créer des plages inversées (ex: 6-2)
      const [startStr, endStr] = part.split('-');
      const start = Number(startStr);
      const end = Number(endStr);

      if (!isNaN(start) && !isNaN(end)) {
        let current = start;
        let safeCount = 0;
        while (safeCount++ <= rangeSize) {
          resultParts.push(String(shiftVal(current)));
          if (current === end) break;
          current++;
          if (current > max) current = min; // Wrap-around pour la lecture de la plage
        }
      } else {
        resultParts.push(part); // Fallback si texte
      }
    } else {
      // Chiffre simple ou expression complexe (ex: */2)
      const num = Number(part);
      if (!isNaN(num)) {
        resultParts.push(String(shiftVal(num)));
      } else {
        resultParts.push(part); // On laisse intact ce qu'on ne sait pas parser
      }
    }
  }

  // Dédoublonner et trier proprement (si ce ne sont que des nombres)
  const uniqueParts = Array.from(new Set(resultParts));
  if (uniqueParts.every(p => !isNaN(Number(p)))) {
    uniqueParts.sort((a, b) => Number(a) - Number(b));
  }

  return uniqueParts.join(',');
}

/**
 * Logique partagée pour la conversion de fuseau horaire
 */
function convertCronTimezone(cronExpression: string, toUTC: boolean): string {
  const fields = cronExpression.trim().split(/\s+/);
  if (fields.length !== 5) return cronExpression;

  const [minute, hour, dayOfMonth, month, dayOfWeek] = fields;

  if (isNaN(Number(minute)) || isNaN(Number(hour))) {
    return cronExpression;
  }

  const now = new Date();
  let targetMinute: string;
  let targetHour: string;
  let dayDiff = 0;

  if (toUTC) {
    // 1. Création de la date locale pour récupérer l'UTC
    const localDate = new Date(
      now.getFullYear(),
      now.getMonth(),
      now.getDate(),
      Number(hour),
      Number(minute),
    );
    targetMinute = String(localDate.getUTCMinutes());
    targetHour = String(localDate.getUTCHours());

    const localDayNum = localDate.getDate();
    const utcDayNum = localDate.getUTCDate();

    if (utcDayNum !== localDayNum) {
      if (utcDayNum > localDayNum && utcDayNum - localDayNum > 1) dayDiff = -1;
      else if (localDayNum > utcDayNum && localDayNum - utcDayNum > 1) dayDiff = 1;
      else dayDiff = utcDayNum - localDayNum;
    }
  } else {
    // 1. Création de la date UTC pour récupérer le local
    const utcDate = new Date(
      Date.UTC(
        now.getUTCFullYear(),
        now.getUTCMonth(),
        now.getUTCDate(),
        Number(hour),
        Number(minute),
      ),
    );
    targetMinute = String(utcDate.getMinutes());
    targetHour = String(utcDate.getHours());

    const localDayNum = utcDate.getDate();
    const utcDayNum = utcDate.getUTCDate();

    if (localDayNum !== utcDayNum) {
      if (localDayNum > utcDayNum && localDayNum - utcDayNum > 1) dayDiff = -1;
      else if (utcDayNum > localDayNum && utcDayNum - localDayNum > 1) dayDiff = 1;
      else dayDiff = localDayNum - utcDayNum;
    }
  }

  if (dayDiff === 0) {
    return `${targetMinute} ${targetHour} ${dayOfMonth} ${month} ${dayOfWeek}`;
  }

  // 2. Ajustement des jours en utilisant le parseur robuste
  const shiftedDom = shiftCronField(dayOfMonth, dayDiff, 1, 31);
  const shiftedDow = shiftCronField(dayOfWeek, dayDiff, 0, 6); // Supposant 0 = Dimanche, 6 = Samedi

  return `${targetMinute} ${targetHour} ${shiftedDom} ${month} ${shiftedDow}`;
}

/**
 * Convert a Cron 5-field expression from the local timezone to UTC.
 */
export function convertCronToUTC(cronExpression: string): string {
  return convertCronTimezone(cronExpression, true);
}

/**
 * Convert a Cron 5-field expression from UTC to the local timezone.
 * Handles hour shifts and adjusts days (dayOfMonth / dayOfWeek) if the timezone boundary is crossed.
 */
export function convertCronToLocal(cronExpression: string): string {
  return convertCronTimezone(cronExpression, false);
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
    console.log('before', finalCronExpression);
    finalCronExpression = convertCronToUTC(finalCronExpression);
    console.log('after', finalCronExpression);
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
