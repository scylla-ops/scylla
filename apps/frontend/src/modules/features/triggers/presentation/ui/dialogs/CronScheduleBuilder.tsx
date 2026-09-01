import { useEffect, useState, type ReactNode } from 'react';
import {
  Input,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@shadcn';
import { RadioGroup, RadioGroupItem } from '@shadcn/radio-group.tsx';
import { ToggleGroup, ToggleGroupItem } from '@shadcn/toggle-group.tsx';
import { Trans } from '@lingui/react/macro';
import { useLingui } from '@lingui/react';
import { cn } from '@shared/presentation/utils';
import {
  buildCron,
  describeCron,
  pad2,
  parseCron,
  WEEKDAYS,
  type CronFrequency,
  type CronModel,
} from '@/modules/features/triggers/presentation/utils/cron.utils.ts';

interface CronScheduleBuilderProps {
  /** Initial cron expression (read once on mount). */
  initialValue: string;
  /** Emits the rendered cron string whenever the schedule changes. */
  onChange: (expression: string) => void;
}

const HOURS = Array.from({ length: 24 }, (_, i) => i);
const MINUTES = Array.from({ length: 60 }, (_, i) => i);
const DAYS_OF_MONTH = Array.from({ length: 31 }, (_, i) => i + 1);

const FrequencyCard = ({
  value,
  current,
  title,
  description,
  className,
}: {
  value: CronFrequency;
  current: CronFrequency;
  title: ReactNode;
  description: ReactNode;
  className?: string;
}) => (
  <Label
    htmlFor={`freq-${value}`}
    className={cn(
      'flex cursor-pointer items-start gap-2 rounded-md border p-2.5 transition-colors',
      current === value ? 'border-primary bg-primary/5' : 'hover:bg-muted/50',
      className,
    )}
  >
    <RadioGroupItem value={value} id={`freq-${value}`} className='mt-0.5' />
    <div className='flex flex-col'>
      <span className='text-sm font-medium leading-tight'>{title}</span>
      <span className='text-xs text-muted-foreground'>{description}</span>
    </div>
  </Label>
);

/** Compact UTC time picker (hour : minute). */
const TimePicker = ({
  hour,
  minute,
  onChange,
}: {
  hour: number;
  minute: number;
  onChange: (next: { hour: number; minute: number }) => void;
}) => (
  <div className='flex items-center gap-2'>
    <span className='text-sm text-muted-foreground'>
      <Trans>at</Trans>
    </span>
    <Select value={String(hour)} onValueChange={v => onChange({ hour: Number(v), minute })}>
      <SelectTrigger className='w-[72px]'>
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {HOURS.map(h => (
          <SelectItem key={h} value={String(h)}>
            {pad2(h)}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
    <span className='text-muted-foreground'>:</span>
    <Select value={String(minute)} onValueChange={v => onChange({ hour, minute: Number(v) })}>
      <SelectTrigger className='w-[72px]'>
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {MINUTES.map(m => (
          <SelectItem key={m} value={String(m)}>
            {pad2(m)}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
    <span className='text-xs text-muted-foreground'>UTC</span>
  </div>
);

/**
 * A visual schedule builder: pick a frequency and the relevant when, instead of
 * hand-writing cron. The expression is shown live so power users still see it.
 */
export const CronScheduleBuilder = ({ initialValue, onChange }: CronScheduleBuilderProps) => {
  const { _, i18n } = useLingui();
  const [model, setModel] = useState<CronModel>(() => parseCron(initialValue));

  // Single source of truth: re-emit the rendered cron whenever the model changes
  // (including on mount, so a freshly created trigger gets a valid default).
  useEffect(() => {
    onChange(buildCron(model));
  }, [model, onChange]);

  const setFrequency = (frequency: CronFrequency) =>
    setModel(prev =>
      frequency === 'custom'
        ? { ...prev, frequency, custom: prev.custom || buildCron(prev) }
        : { ...prev, frequency },
    );

  return (
    <div className='space-y-3'>
      <RadioGroup
        value={model.frequency}
        onValueChange={value => setFrequency(value as CronFrequency)}
        className='grid grid-cols-2 gap-2'
      >
        <FrequencyCard
          value='hourly'
          current={model.frequency}
          title={<Trans>Hourly</Trans>}
          description={<Trans>Every hour</Trans>}
        />
        <FrequencyCard
          value='daily'
          current={model.frequency}
          title={<Trans>Daily</Trans>}
          description={<Trans>Every day</Trans>}
        />
        <FrequencyCard
          value='weekly'
          current={model.frequency}
          title={<Trans>Weekly</Trans>}
          description={<Trans>On chosen days</Trans>}
        />
        <FrequencyCard
          value='monthly'
          current={model.frequency}
          title={<Trans>Monthly</Trans>}
          description={<Trans>On a day of the month</Trans>}
        />
        <FrequencyCard
          value='custom'
          current={model.frequency}
          title={<Trans>Custom</Trans>}
          description={<Trans>Write a cron expression</Trans>}
          className='col-span-2'
        />
      </RadioGroup>

      {/* Contextual controls per frequency */}
      {model.frequency === 'hourly' && (
        <div className='flex items-center gap-2'>
          <span className='text-sm text-muted-foreground'>
            <Trans>at minute</Trans>
          </span>
          <Select
            value={String(model.minute)}
            onValueChange={v => setModel(prev => ({ ...prev, minute: Number(v) }))}
          >
            <SelectTrigger className='w-[72px]'>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {MINUTES.map(m => (
                <SelectItem key={m} value={String(m)}>
                  {pad2(m)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      )}

      {model.frequency === 'daily' && (
        <TimePicker
          hour={model.hour}
          minute={model.minute}
          onChange={t => setModel(prev => ({ ...prev, ...t }))}
        />
      )}

      {model.frequency === 'weekly' && (
        <div className='space-y-2'>
          <ToggleGroup
            type='multiple'
            variant='outline'
            value={model.weekdays.map(String)}
            onValueChange={values => setModel(prev => ({ ...prev, weekdays: values.map(Number) }))}
          >
            {WEEKDAYS.map(day => (
              <ToggleGroupItem key={day.value} value={String(day.value)} className='px-2.5'>
                {_(day.label)}
              </ToggleGroupItem>
            ))}
          </ToggleGroup>
          <TimePicker
            hour={model.hour}
            minute={model.minute}
            onChange={t => setModel(prev => ({ ...prev, ...t }))}
          />
        </div>
      )}

      {model.frequency === 'monthly' && (
        <div className='flex flex-wrap items-center gap-2'>
          <span className='text-sm text-muted-foreground'>
            <Trans>on day</Trans>
          </span>
          <Select
            value={String(model.dayOfMonth)}
            onValueChange={v => setModel(prev => ({ ...prev, dayOfMonth: Number(v) }))}
          >
            <SelectTrigger className='w-[72px]'>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {DAYS_OF_MONTH.map(d => (
                <SelectItem key={d} value={String(d)}>
                  {d}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <TimePicker
            hour={model.hour}
            minute={model.minute}
            onChange={t => setModel(prev => ({ ...prev, ...t }))}
          />
        </div>
      )}

      {model.frequency === 'custom' && (
        <div className='space-y-1'>
          <Input
            value={model.custom}
            placeholder='*/15 * * * *'
            onChange={e => setModel(prev => ({ ...prev, custom: e.target.value }))}
            className='font-mono'
          />
          <p className='text-xs text-muted-foreground'>
            <Trans>5-field cron (min hour day month weekday), evaluated in your local time.</Trans>
          </p>
        </div>
      )}

      {/* Live summary + the resulting expression */}
      <div className='flex flex-wrap items-center gap-2 rounded-md bg-muted/50 px-3 py-2'>
        <span className='text-xs text-muted-foreground'>{describeCron(model, i18n)}</span>
        <code className='ml-auto font-mono text-xs text-foreground'>{buildCron(model) || '—'}</code>
      </div>
    </div>
  );
};
