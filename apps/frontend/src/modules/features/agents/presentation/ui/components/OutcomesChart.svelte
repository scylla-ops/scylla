<script lang="ts">
  import { i18n } from '@lingui/core';
  import { Card, CardContent } from '@shadcn';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { DailyOutcome } from '../../../domain/structs/agent.struct.ts';
  import {
    bucketTotal,
    bucketsMax,
    fillBuckets,
    OUTCOME_RANGES,
    type Bucket,
    type OutcomeRange,
  } from './outcomes-chart.calculator.ts';
  import { agentsMessages } from '../agents.messages.ts';

  interface Props {
    daily: DailyOutcome[];
    aggregate: { completed: number; failed: number; cancelled: number };
  }

  let { daily, aggregate }: Props = $props();

  // Like STATUS_CONFIG: completed uses the primary color.
  const SEGMENTS = [
    { key: 'completed', color: 'var(--primary)' },
    { key: 'failed', color: 'var(--destructive)' },
    { key: 'cancelled', color: 'var(--warning)' },
  ] as const;

  let range = $state<OutcomeRange>('14d');
  let hover = $state<number | null>(null);

  const buckets = $derived(fillBuckets(daily, range));
  const windowTotal = $derived(buckets.reduce((sum, bucket) => sum + bucketTotal(bucket), 0));
  const max = $derived(bucketsMax(buckets));

  const monthLabel = $derived(
    buckets.length ? new Date(buckets[0].day).toLocaleString(i18n.locale, { month: 'short' }) : '',
  );

  const segmentValue = (bucket: Bucket, key: (typeof SEGMENTS)[number]['key']) => bucket[key];
</script>

<Card class="w-full">
  <CardContent class="space-y-3 p-4 w-full">
    <div class="flex items-center justify-between">
      <span class="font-mono text-xs uppercase tracking-wide text-muted-foreground">
        {t(agentsMessages.outcomesLast)}
        {range}
      </span>
      <div class="flex overflow-hidden rounded border text-[11px] font-mono">
        {#each OUTCOME_RANGES as option (option)}
          <button
            type="button"
            onclick={() => (range = option)}
            aria-pressed={option === range}
            class={cn(
              'px-2 py-0.5 transition-colors',
              option === range
                ? 'bg-foreground text-background'
                : 'text-muted-foreground hover:bg-muted',
            )}
          >
            {option}
          </button>
        {/each}
      </div>
    </div>

    {#if windowTotal === 0}
      <div class="flex h-[140px] flex-col items-center justify-center gap-1 text-center">
        <p class="text-sm text-muted-foreground">{t(agentsMessages.noFinishedJobs)}</p>
        <p class="text-xs text-muted-foreground/70">{t(agentsMessages.runAPipeline)}</p>
      </div>
    {:else}
      <div class="relative pl-7">
        <div
          class="pointer-events-none absolute left-0 top-0 flex h-[120px] w-6 flex-col justify-between font-mono text-[9px] text-muted-foreground"
        >
          <span>{max}</span>
          <span>{Math.round(max / 2)}</span>
          <span>0</span>
        </div>

        <div class="relative h-[120px] border-b border-foreground/80">
          <div class="pointer-events-none absolute inset-0 flex flex-col justify-between">
            <div class="border-t border-dashed border-border"></div>
            <div class="border-t border-dashed border-border"></div>
            <div class="border-t border-transparent"></div>
          </div>

          <div class="flex h-full items-end gap-px">
            {#each buckets as bucket, index (bucket.day)}
              {@const total = bucketTotal(bucket)}
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="group relative flex h-full flex-1 flex-col justify-end"
                title={t(
                  agentsMessages.bucketSummary(
                    bucket.day,
                    bucket.completed,
                    bucket.failed,
                    bucket.cancelled,
                  ),
                )}
                onmouseenter={() => (hover = index)}
                onmouseleave={() => {
                  if (hover === index) hover = null;
                }}
              >
                <!-- The bars' percentage heights need `h-full` on the wrapper above. -->
                <div
                  class={cn(
                    'mx-auto flex w-full max-w-[28px] flex-col-reverse transition-opacity',
                    hover === null || hover === index ? 'opacity-100' : 'opacity-60',
                  )}
                  style={`height: ${(total / max) * 100}%`}
                >
                  {#each SEGMENTS as segment (segment.key)}
                    {@const value = segmentValue(bucket, segment.key)}
                    {#if value}
                      <div
                        style={`height: ${(value / total) * 100}%; background: ${segment.color}`}
                      ></div>
                    {/if}
                  {/each}
                </div>

                {#if hover === index}
                  <div
                    class="pointer-events-none absolute -top-2 left-1/2 z-10 w-max -translate-x-1/2 -translate-y-full rounded bg-foreground px-2 py-1.5 text-[10px] text-background shadow-md"
                  >
                    <p class="font-semibold">
                      {bucket.day} · {total}
                      {t(agentsMessages.runs)}
                    </p>
                    {#if bucket.completed > 0}
                      <p>
                        <span style="color: var(--primary)">●</span>
                        {t(agentsMessages.completedCount(bucket.completed))}
                      </p>
                    {/if}
                    {#if bucket.failed > 0}
                      <p>
                        <span style="color: var(--destructive)">●</span>
                        {t(agentsMessages.failedCount(bucket.failed))}
                      </p>
                    {/if}
                    {#if bucket.cancelled > 0}
                      <p>
                        <span style="color: var(--warning)">●</span>
                        {t(agentsMessages.cancelledCount(bucket.cancelled))}
                      </p>
                    {/if}
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        </div>

        <div class="mt-1 flex gap-px pl-0">
          <span class="absolute left-0 font-mono text-[9px] text-muted-foreground">
            {monthLabel}
          </span>
          {#each buckets as bucket, index (bucket.day)}
            <span
              class={cn(
                'flex-1 text-center font-mono text-[9px]',
                hover === index ? 'text-foreground' : 'text-muted-foreground',
              )}
            >
              {buckets.length > 16 && index % 2 === 1 && hover !== index
                ? ''
                : new Date(bucket.day).getDate()}
            </span>
          {/each}
        </div>
      </div>
    {/if}

    <div class="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs">
      <span class="flex items-center gap-1.5">
        <span class="h-2.5 w-2.5 rounded-sm" style="background: var(--primary)"></span>
        <span class="font-semibold">{aggregate.completed}</span>
        {t(agentsMessages.completed)}
      </span>
      <span class="flex items-center gap-1.5">
        <span class="h-2.5 w-2.5 rounded-sm" style="background: var(--destructive)"></span>
        <span class="font-semibold">{aggregate.failed}</span>
        {t(agentsMessages.failed)}
      </span>
      <span class="flex items-center gap-1.5">
        <span class="h-2.5 w-2.5 rounded-sm" style="background: var(--warning)"></span>
        <span class="font-semibold">{aggregate.cancelled}</span>
        {t(agentsMessages.cancelled)}
      </span>
      <span class="ml-auto font-mono text-[11px] text-muted-foreground">
        {aggregate.completed + aggregate.failed + aggregate.cancelled}
        {t(agentsMessages.finished)}
      </span>
    </div>
  </CardContent>
</Card>
