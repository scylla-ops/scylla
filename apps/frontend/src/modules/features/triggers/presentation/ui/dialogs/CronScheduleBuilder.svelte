<script lang="ts">
  import { i18n } from '@lingui/core';
  import {
    Input,
    Label,
    RadioGroup,
    RadioGroupItem,
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
    ToggleGroup,
    ToggleGroupItem,
  } from '@shadcn-svelte';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import {
    buildCron,
    describeCron,
    pad2,
    parseCron,
    WEEKDAYS,
    type CronFrequency,
    type CronModel,
  } from '../../utils/cron.utils.ts';
  import { triggersMessages } from '../triggers.messages.ts';

  interface Props {
    /** Initial cron expression (read once on mount). */
    initialValue: string;
    /** Emits the rendered cron string whenever the schedule changes. */
    onChange: (expression: string) => void;
  }

  let { initialValue, onChange }: Props = $props();

  const HOURS = Array.from({ length: 24 }, (_, index) => index);
  const MINUTES = Array.from({ length: 60 }, (_, index) => index);
  const DAYS_OF_MONTH = Array.from({ length: 31 }, (_, index) => index + 1);

  // Read once, deliberately: the prop is documented as the *initial* value and
  // the parent recreates this component when the dialog reopens.
  // svelte-ignore state_referenced_locally
  let model = $state<CronModel>(parseCron(initialValue));

  const expression = $derived(buildCron(model));

  /**
   * One source of truth: the expression is *derived* from the model, and every
   * write goes through here so the parent is told at the same moment.
   *
   * The React version ran an effect on `[model, onChange]` to re-emit, which
   * also fired on mount — that mattered, because a freshly created trigger needs
   * a valid default. `parseCron(initialValue)` already gives one, and the parent
   * seeds its own state from the same string, so nothing is lost by not firing
   * before the user has touched anything.
   */
  const patch = (next: Partial<CronModel>) => {
    model = { ...model, ...next };
    onChange(buildCron(model));
  };

  const setFrequency = (frequency: CronFrequency) =>
    patch(
      frequency === 'custom'
        ? { frequency, custom: model.custom || buildCron(model) }
        : { frequency },
    );

  const FREQUENCIES: Array<{ value: CronFrequency; title: string; description: string; wide?: boolean }> =
    $derived([
      { value: 'hourly', title: t(triggersMessages.hourly), description: t(triggersMessages.everyHour) },
      { value: 'daily', title: t(triggersMessages.daily), description: t(triggersMessages.everyDay) },
      { value: 'weekly', title: t(triggersMessages.weekly), description: t(triggersMessages.onChosenDays) },
      { value: 'monthly', title: t(triggersMessages.monthly), description: t(triggersMessages.onADayOfTheMonth) },
      { value: 'custom', title: t(triggersMessages.custom), description: t(triggersMessages.writeACron), wide: true },
    ]);
</script>

{#snippet numberSelect(label: string, value: number, options: number[], onPick: (n: number) => void, format: (n: number) => string)}
  <Select type="single" value={String(value)} onValueChange={picked => onPick(Number(picked))}>
    <SelectTrigger class="w-[72px]" aria-label={label}>
      <SelectValue />
    </SelectTrigger>
    <SelectContent>
      {#each options as option (option)}
        <SelectItem value={String(option)} label={format(option)}>{format(option)}</SelectItem>
      {/each}
    </SelectContent>
  </Select>
{/snippet}

<!-- Compact UTC time picker (hour : minute). -->
{#snippet timePicker()}
  <div class="flex items-center gap-2">
    <span class="text-sm text-muted-foreground">{t(triggersMessages.at)}</span>
    {@render numberSelect(
      t(triggersMessages.hour),
      model.hour,
      HOURS,
      hour => patch({ hour }),
      pad2,
    )}
    <span class="text-muted-foreground">:</span>
    {@render numberSelect(
      t(triggersMessages.minute),
      model.minute,
      MINUTES,
      minute => patch({ minute }),
      pad2,
    )}
    <span class="text-xs text-muted-foreground">{t(triggersMessages.localTime)}</span>
  </div>
{/snippet}

<!--
  A visual schedule builder: pick a frequency and the relevant when, instead of
  hand-writing cron. The expression is shown live so power users still see it.
-->
<div class="space-y-3">
  <RadioGroup
    value={model.frequency}
    onValueChange={value => setFrequency(value as CronFrequency)}
    class="grid grid-cols-2 gap-2"
    aria-label={t(triggersMessages.frequency)}
  >
    {#each FREQUENCIES as frequency (frequency.value)}
      <Label
        for={`freq-${frequency.value}`}
        class={cn(
          'flex cursor-pointer items-start gap-2 rounded-md border p-2.5 transition-colors',
          model.frequency === frequency.value
            ? 'border-primary bg-primary/5'
            : 'hover:bg-muted/50',
          frequency.wide && 'col-span-2',
        )}
      >
        <RadioGroupItem value={frequency.value} id={`freq-${frequency.value}`} class="mt-0.5" />
        <div class="flex flex-col">
          <span class="text-sm font-medium leading-tight">{frequency.title}</span>
          <span class="text-xs text-muted-foreground">{frequency.description}</span>
        </div>
      </Label>
    {/each}
  </RadioGroup>

  <!-- Contextual controls per frequency -->
  {#if model.frequency === 'hourly'}
    <div class="flex items-center gap-2">
      <span class="text-sm text-muted-foreground">{t(triggersMessages.atMinute)}</span>
      {@render numberSelect(
        t(triggersMessages.minute),
        model.minute,
        MINUTES,
        minute => patch({ minute }),
        pad2,
      )}
    </div>
  {:else if model.frequency === 'daily'}
    {@render timePicker()}
  {:else if model.frequency === 'weekly'}
    <div class="space-y-2">
      <ToggleGroup
        type="multiple"
        variant="outline"
        value={model.weekdays.map(String)}
        onValueChange={values => patch({ weekdays: values.map(Number) })}
        aria-label={t(triggersMessages.weekdays)}
      >
        {#each WEEKDAYS as day (day.value)}
          <ToggleGroupItem value={String(day.value)} class="px-2.5">
            {i18n._(day.label)}
          </ToggleGroupItem>
        {/each}
      </ToggleGroup>
      {@render timePicker()}
    </div>
  {:else if model.frequency === 'monthly'}
    <div class="flex flex-wrap items-center gap-2">
      <span class="text-sm text-muted-foreground">{t(triggersMessages.onDay)}</span>
      {@render numberSelect(
        t(triggersMessages.dayOfMonth),
        model.dayOfMonth,
        DAYS_OF_MONTH,
        dayOfMonth => patch({ dayOfMonth }),
        String,
      )}
      {@render timePicker()}
    </div>
  {:else if model.frequency === 'custom'}
    <div class="space-y-1">
      <Input
        value={model.custom}
        placeholder="*/15 * * * *"
        oninput={event => patch({ custom: event.currentTarget.value })}
        class="font-mono"
        aria-label={t(triggersMessages.custom)}
      />
      <p class="text-xs text-muted-foreground">{t(triggersMessages.cronHint)}</p>
    </div>
  {/if}

  <!-- Live summary + the resulting expression -->
  <div class="flex flex-wrap items-center gap-2 rounded-md bg-muted/50 px-3 py-2">
    <span class="text-xs text-muted-foreground">{describeCron(model, i18n)}</span>
    <code class="ml-auto font-mono text-xs text-foreground">{expression || '—'}</code>
  </div>
</div>
