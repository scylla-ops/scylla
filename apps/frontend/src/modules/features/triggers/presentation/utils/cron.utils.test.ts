import { describe, it, expect } from 'vitest';
import { i18n } from '@lingui/core';
import { buildCron, parseCron, describeCron, pad2, type CronModel } from './cron.utils';

const base: Omit<CronModel, 'frequency'> = {
  minute: 0,
  hour: 9,
  weekdays: [1, 2, 3, 4, 5],
  dayOfMonth: 1,
  custom: '',
};

describe('pad2', () => {
  it('pads single digits with a leading zero', () => {
    expect(pad2(5)).toBe('05');
  });

  it('leaves two-digit numbers untouched', () => {
    expect(pad2(23)).toBe('23');
  });
});

describe('buildCron', () => {
  it('builds an hourly expression', () => {
    expect(buildCron({ ...base, frequency: 'hourly', minute: 15 })).toBe('15 * * * *');
  });

  it('builds a daily expression', () => {
    expect(buildCron({ ...base, frequency: 'daily', minute: 30, hour: 14 })).toBe('30 14 * * *');
  });

  it('builds a weekly expression with sorted, deduplicated weekdays', () => {
    expect(
      buildCron({ ...base, frequency: 'weekly', minute: 0, hour: 9, weekdays: [5, 1, 3] }),
    ).toBe('0 9 * * 1,3,5');
  });

  it('falls back to * for weekly with no weekdays selected', () => {
    expect(buildCron({ ...base, frequency: 'weekly', weekdays: [] })).toBe('0 9 * * *');
  });

  it('builds a monthly expression', () => {
    expect(buildCron({ ...base, frequency: 'monthly', minute: 0, hour: 3, dayOfMonth: 15 })).toBe(
      '0 3 15 * *',
    );
  });

  it('passes a custom expression through trimmed', () => {
    expect(buildCron({ ...base, frequency: 'custom', custom: '  */5 * * * *  ' })).toBe(
      '*/5 * * * *',
    );
  });
});

describe('parseCron', () => {
  it('round-trips every built shape back to an equivalent model', () => {
    const models: CronModel[] = [
      { ...base, frequency: 'hourly', minute: 15 },
      { ...base, frequency: 'daily', minute: 30, hour: 14 },
      { ...base, frequency: 'weekly', minute: 0, hour: 9, weekdays: [1, 3, 5] },
      { ...base, frequency: 'monthly', minute: 0, hour: 3, dayOfMonth: 15 },
    ];

    for (const model of models) {
      const parsed = parseCron(buildCron(model));
      expect(parsed.frequency).toBe(model.frequency);
      expect(parsed.minute).toBe(model.minute);
      if (model.frequency === 'weekly') {
        expect(parsed.weekdays).toEqual(model.weekdays);
      }
    }
  });

  it('treats an empty expression as the default model', () => {
    expect(parseCron('')).toEqual({ ...base, frequency: 'daily' });
    expect(parseCron('   ')).toEqual({ ...base, frequency: 'daily' });
  });

  it('falls back to custom for a non-5-field expression', () => {
    const parsed = parseCron('0 9 * *');
    expect(parsed.frequency).toBe('custom');
    expect(parsed.custom).toBe('0 9 * *');
  });

  it('falls back to custom when the month field is anything but *', () => {
    const parsed = parseCron('0 9 1 6 *');
    expect(parsed.frequency).toBe('custom');
  });

  it('falls back to custom for a step expression like */5 * * * *', () => {
    const parsed = parseCron('*/5 * * * *');
    expect(parsed.frequency).toBe('custom');
    expect(parsed.custom).toBe('*/5 * * * *');
  });

  it('rejects an out-of-range weekday and falls back to custom', () => {
    const parsed = parseCron('0 9 * * 1,9');
    expect(parsed.frequency).toBe('custom');
  });
});

describe('describeCron', () => {
  it('describes an hourly schedule', () => {
    expect(describeCron({ ...base, frequency: 'hourly', minute: 5 }, i18n)).toBe(
      'Every hour at :05',
    );
  });

  it('describes a daily schedule with zero-padded time', () => {
    expect(describeCron({ ...base, frequency: 'daily', minute: 5, hour: 9 }, i18n)).toBe(
      'Every day at 09:05',
    );
  });

  it('describes a weekly schedule by weekday labels, Mon-first order', () => {
    expect(
      describeCron({ ...base, frequency: 'weekly', minute: 0, hour: 9, weekdays: [0, 1] }, i18n),
    ).toBe('Mon, Sun at 09:00');
  });

  it('flags an empty weekly selection as needing at least one day', () => {
    expect(describeCron({ ...base, frequency: 'weekly', weekdays: [] }, i18n)).toBe(
      'Pick at least one day',
    );
  });

  it('describes a monthly schedule', () => {
    expect(
      describeCron({ ...base, frequency: 'monthly', minute: 0, hour: 3, dayOfMonth: 15 }, i18n),
    ).toBe('Day 15 of each month at 03:00');
  });

  it('describes a filled-in custom schedule generically', () => {
    expect(describeCron({ ...base, frequency: 'custom', custom: '*/5 * * * *' }, i18n)).toBe(
      'Custom schedule',
    );
  });

  it('describes an empty custom schedule as incomplete', () => {
    expect(describeCron({ ...base, frequency: 'custom', custom: '' }, i18n)).toBe(
      'Incomplete schedule',
    );
  });
});
