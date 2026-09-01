import { msg } from '@lingui/core/macro';
import type { I18n, MessageDescriptor } from '@lingui/core';

/**
 * A friendly, structured view of a 5-field cron expression (UTC). The builder UI
 * edits this model; {@link buildCron} renders it back to a cron string and
 * {@link parseCron} recognizes the common shapes we produce (anything else is
 * kept verbatim as a `custom` expression).
 */
export type CronFrequency = 'hourly' | 'daily' | 'weekly' | 'monthly' | 'custom';

export interface CronModel {
  frequency: CronFrequency;
  minute: number; // 0-59
  hour: number; // 0-23
  weekdays: number[]; // cron day-of-week, 0 = Sunday … 6 = Saturday
  dayOfMonth: number; // 1-31
  custom: string; // raw expression, for `custom`
}

/**
 * Weekday chips, displayed Mon→Sun; `value` is the cron day-of-week number.
 * Labels are lazy messages — this module is imported outside any i18n context.
 */
export const WEEKDAYS: { value: number; label: MessageDescriptor }[] = [
  { value: 1, label: msg`Mon` },
  { value: 2, label: msg`Tue` },
  { value: 3, label: msg`Wed` },
  { value: 4, label: msg`Thu` },
  { value: 5, label: msg`Fri` },
  { value: 6, label: msg`Sat` },
  { value: 0, label: msg`Sun` },
];

const DEFAULT_MODEL: CronModel = {
  frequency: 'daily',
  minute: 0,
  hour: 9,
  weekdays: [1, 2, 3, 4, 5],
  dayOfMonth: 1,
  custom: '',
};

export const pad2 = (n: number): string => n.toString().padStart(2, '0');
const isInt = (s: string): boolean => /^\d+$/.test(s);

export const buildCron = (model: CronModel): string => {
  switch (model.frequency) {
    case 'hourly':
      return `${model.minute} * * * *`;
    case 'daily':
      return `${model.minute} ${model.hour} * * *`;
    case 'weekly': {
      const dow = model.weekdays.length
        ? [...model.weekdays].sort((a, b) => a - b).join(',')
        : '*';
      return `${model.minute} ${model.hour} * * ${dow}`;
    }
    case 'monthly':
      return `${model.minute} ${model.hour} ${model.dayOfMonth} * *`;
    case 'custom':
      return model.custom.trim();
  }
};

/** Best-effort: recognize the shapes the builder produces, else fall back to custom. */
export const parseCron = (expression: string): CronModel => {
  const trimmed = expression.trim();
  if (trimmed.length === 0) return { ...DEFAULT_MODEL };

  const toCustom = (): CronModel => ({ ...DEFAULT_MODEL, frequency: 'custom', custom: trimmed });

  const parts = trimmed.split(/\s+/);
  if (parts.length !== 5) return toCustom();

  const [min, hr, dom, mon, dow] = parts;
  if (mon !== '*') return toCustom();

  if (isInt(min) && hr === '*' && dom === '*' && dow === '*') {
    return { ...DEFAULT_MODEL, frequency: 'hourly', minute: Number(min) };
  }
  if (isInt(min) && isInt(hr) && dom === '*' && dow === '*') {
    return { ...DEFAULT_MODEL, frequency: 'daily', minute: Number(min), hour: Number(hr) };
  }
  if (isInt(min) && isInt(hr) && dom === '*' && dow !== '*') {
    const days = dow.split(',');
    if (days.every(d => isInt(d) && Number(d) >= 0 && Number(d) <= 6)) {
      return {
        ...DEFAULT_MODEL,
        frequency: 'weekly',
        minute: Number(min),
        hour: Number(hr),
        weekdays: days.map(Number),
      };
    }
  }
  if (isInt(min) && isInt(hr) && isInt(dom) && dow === '*') {
    return {
      ...DEFAULT_MODEL,
      frequency: 'monthly',
      minute: Number(min),
      hour: Number(hr),
      dayOfMonth: Number(dom),
    };
  }
  return toCustom();
};

/**
 * A plain-language summary of the schedule. Takes the active `i18n` because it
 * runs outside the component tree — the caller passes the one from `useLingui`.
 */
export const describeCron = (model: CronModel, i18n: I18n): string => {
  const time = `${pad2(model.hour)}:${pad2(model.minute)} UTC`;
  switch (model.frequency) {
    case 'hourly':
      return i18n._(msg`Every hour at :${pad2(model.minute)}`);
    case 'daily':
      return i18n._(msg`Every day at ${time}`);
    case 'weekly': {
      if (model.weekdays.length === 0) return i18n._(msg`Pick at least one day`);
      const days = WEEKDAYS.filter(d => model.weekdays.includes(d.value))
        .map(d => i18n._(d.label))
        .join(', ');
      return i18n._(msg`${days} at ${time}`);
    }
    case 'monthly':
      return i18n._(msg`Day ${model.dayOfMonth} of each month at ${time}`);
    case 'custom':
      return model.custom.trim() ? i18n._(msg`Custom schedule`) : i18n._(msg`Incomplete schedule`);
  }
};
