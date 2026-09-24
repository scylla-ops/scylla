<script lang="ts">
  import { contextStore } from '@platform/context';
  import { createQuery } from '@platform/query';
  import { Card, CardContent, CardHeader, CardTitle, Skeleton } from '@shadcn';
  import { toRune } from '@shared/presentation/stores/to-rune.svelte.ts';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { agentQueries } from '@/modules/features/agents';
  import {
    fillBuckets,
    hasActivity,
    labelVisibility,
    monotoneAreaPath,
    monotoneLinePath,
    niceTicks,
    peakOf,
    projectPoints,
    type ChartGeometry,
    type OutcomeSeries,
  } from '../outcomes-chart.calculator.ts';
  import { dashboardMessages } from '../dashboard.messages.ts';

  type Range = '7d' | '14d' | '30d';
  type StatusFilter = 'all' | OutcomeSeries;

  const RANGES: Range[] = ['7d', '14d', '30d'];
  const RANGE_DAYS: Record<Range, number> = { '7d': 7, '14d': 14, '30d': 30 };
  const STATUS_FILTERS: StatusFilter[] = ['all', 'completed', 'failed', 'cancelled'];

  /** The drawing box, in SVG user units — see the note on `preserveAspectRatio`. */
  const GEOMETRY: ChartGeometry = {
    width: 600,
    height: 160,
    padding: { top: 6, right: 2, bottom: 6, left: 2 },
  };

  const SERIES: { key: OutcomeSeries; color: string }[] = [
    { key: 'completed', color: 'var(--success)' },
    { key: 'failed', color: 'var(--destructive)' },
    { key: 'cancelled', color: 'var(--warning)' },
  ];

  const context = toRune(contextStore);
  const organizationId = $derived(context().organization.id ?? '');

  const agentsQuery = createQuery(() => agentQueries.byOrganization(organizationId));
  const agents = $derived(agentsQuery.data ?? []);

  let selectedAgentId = $state<string | null>(null);
  const agentId = $derived(selectedAgentId ?? agents[0]?.id ?? null);

  let range = $state<Range>('14d');
  let status = $state<StatusFilter>('all');
  let hover = $state<number | null>(null);

  const statsQuery = createQuery(() => agentQueries.statsOf(agentId ?? ''));

  const buckets = $derived(fillBuckets(statsQuery.data?.daily ?? [], RANGE_DAYS[range]));
  const shownSeries = $derived(
    status === 'all' ? SERIES : SERIES.filter(series => series.key === status),
  );
  const ticks = $derived(niceTicks(peakOf(buckets, shownSeries.map(series => series.key))));
  const axisTop = $derived(ticks[ticks.length - 1]);
  const showLabel = $derived(labelVisibility(buckets.length));

  const statusLabel = (filter: StatusFilter) =>
    t(
      {
        all: dashboardMessages.filterAll,
        completed: dashboardMessages.filterCompleted,
        failed: dashboardMessages.filterFailed,
        cancelled: dashboardMessages.filterCancelled,
      }[filter],
    );

  const seriesLabel = (key: OutcomeSeries) =>
    t(
      {
        completed: dashboardMessages.completed,
        failed: dashboardMessages.failed,
        cancelled: dashboardMessages.cancelled,
      }[key],
    );

  const pathsOf = (key: OutcomeSeries) => {
    const points = projectPoints(
      buckets.map(bucket => bucket[key]),
      axisTop,
      GEOMETRY,
    );
    return {
      line: monotoneLinePath(points),
      area: monotoneAreaPath(points, GEOMETRY.height - GEOMETRY.padding.bottom),
    };
  };
</script>

<!--
  Run outcomes per agent, over a window — the one chart `recharts` used to draw,
  and the reason 119 kB gzip of it (plus `d3-*` and `victory-vendor`) left the
  bundle in Phase 4.

  The geometry lives in `outcomes-chart.calculator.ts`, tested without a DOM;
  what is left here is markup. Two decisions that matter:

  - **The SVG is stretched, the text is not.** `preserveAspectRatio="none"` lets
    the curve fill whatever width the card has without measuring it, and
    `vector-effect="non-scaling-stroke"` keeps the stroke one pixel wide under
    that stretch. Anything with glyphs — axis ticks, labels, tooltip — is HTML
    beside the SVG rather than `<text>` inside it, which would be distorted by
    the same transform.
  - **Hover is an HTML band per day**, not a point hit-test: the reader is
    asking about a day, and a band is a target the whole height of the chart.
-->
{#if agentsQuery.isLoading}
  <Card><CardContent class="p-4"><Skeleton class="h-[210px] w-full" /></CardContent></Card>
{:else if agents.length === 0}
  <Card>
    <CardContent class="flex h-[140px] items-center justify-center p-4">
      <p class="text-sm text-muted-foreground">{t(dashboardMessages.noAgents)}</p>
    </CardContent>
  </Card>
{:else}
  <Card>
    <CardHeader class="pb-2 px-4 pt-4">
      <div class="flex flex-wrap items-center justify-between gap-3">
        <CardTitle class="text-base">{t(dashboardMessages.agentOutcomes)}</CardTitle>
        <div class="flex overflow-hidden rounded border text-[11px] font-mono">
          {#each agents as agent (agent.id)}
            <button
              type="button"
              onclick={() => (selectedAgentId = agent.id)}
              aria-pressed={agent.id === agentId}
              class={cn(
                'flex items-center gap-1.5 px-2.5 py-1 transition-colors',
                agent.id === agentId
                  ? 'bg-foreground text-background'
                  : 'text-muted-foreground hover:bg-muted',
              )}
            >
              <span
                class={cn(
                  'h-1.5 w-1.5 rounded-full',
                  agent.connected ? 'bg-success' : 'bg-muted-foreground/50',
                )}
              ></span>
              {agent.name}
            </button>
          {/each}
        </div>
      </div>
    </CardHeader>

    <CardContent class="px-4 pb-4">
      <div class="space-y-3">
        <div class="flex flex-wrap items-center justify-between gap-2">
          <div class="flex overflow-hidden rounded border text-[11px] font-mono">
            {#each STATUS_FILTERS as filter (filter)}
              <button
                type="button"
                onclick={() => (status = filter)}
                aria-pressed={filter === status}
                class={cn(
                  'px-2.5 py-1 capitalize transition-colors',
                  filter === status
                    ? 'bg-foreground text-background'
                    : 'text-muted-foreground hover:bg-muted',
                )}
              >
                {statusLabel(filter)}
              </button>
            {/each}
          </div>
          <div class="flex overflow-hidden rounded border text-[11px] font-mono">
            {#each RANGES as option (option)}
              <button
                type="button"
                onclick={() => (range = option)}
                aria-pressed={option === range}
                class={cn(
                  'px-2.5 py-1 transition-colors',
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

        {#if statsQuery.isLoading}
          <Skeleton class="h-[160px] w-full" />
        {:else if !hasActivity(buckets)}
          <div class="flex h-[160px] items-center justify-center text-center">
            <p class="text-sm text-muted-foreground">{t(dashboardMessages.noFinishedJobs)}</p>
          </div>
        {:else}
          <div class="relative pl-7">
            <!-- Y ticks, top to bottom -->
            <div
              class="pointer-events-none absolute left-0 top-0 flex h-[160px] w-6 flex-col justify-between text-right font-mono text-[10px] text-muted-foreground"
            >
              {#each [...ticks].reverse() as tick (tick)}
                <span>{tick}</span>
              {/each}
            </div>

            <div class="relative h-[160px]">
              <svg
                viewBox={`0 0 ${GEOMETRY.width} ${GEOMETRY.height}`}
                preserveAspectRatio="none"
                class="h-full w-full overflow-visible"
                role="img"
                aria-label={t(dashboardMessages.agentOutcomes)}
              >
                <defs>
                  {#each SERIES as series (series.key)}
                    <linearGradient id={`grad-${series.key}`} x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stop-color={series.color} stop-opacity="0.25" />
                      <stop offset="95%" stop-color={series.color} stop-opacity="0" />
                    </linearGradient>
                  {/each}
                </defs>

                <!-- Horizontal gridlines, one per tick -->
                {#each ticks as tick (tick)}
                  {@const y =
                    GEOMETRY.padding.top +
                    (GEOMETRY.height - GEOMETRY.padding.top - GEOMETRY.padding.bottom) *
                      (1 - tick / (axisTop || 1))}
                  <line
                    x1="0"
                    x2={GEOMETRY.width}
                    y1={y}
                    y2={y}
                    class="stroke-border"
                    stroke-dasharray="3 3"
                    vector-effect="non-scaling-stroke"
                  />
                {/each}

                {#each shownSeries as series (series.key)}
                  {@const paths = pathsOf(series.key)}
                  <path d={paths.area} fill={`url(#grad-${series.key})`} />
                  <path
                    d={paths.line}
                    fill="none"
                    stroke={series.color}
                    stroke-width="2"
                    vector-effect="non-scaling-stroke"
                  />
                {/each}
              </svg>

              <!-- One hover band per day, over the whole chart height -->
              <div class="absolute inset-0 flex">
                {#each buckets as bucket, index (bucket.day)}
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <div
                    class="relative flex-1"
                    onmouseenter={() => (hover = index)}
                    onmouseleave={() => {
                      if (hover === index) hover = null;
                    }}
                  >
                    {#if hover === index}
                      <div
                        class="pointer-events-none absolute bottom-2 left-1/2 z-10 w-max -translate-x-1/2 rounded bg-foreground px-2 py-1.5 text-[10px] text-background shadow-md"
                      >
                        <p class="font-semibold">{bucket.day}</p>
                        {#each shownSeries as series (series.key)}
                          <p>
                            <span style={`color: ${series.color}`}>●</span>
                            {seriesLabel(series.key)}
                            {bucket[series.key]}
                          </p>
                        {/each}
                      </div>
                    {/if}
                  </div>
                {/each}
              </div>
            </div>

            <!-- X labels — thinned so 30 day-numbers do not smear together -->
            <div class="mt-1 flex">
              {#each buckets as bucket, index (bucket.day)}
                <span
                  class={cn(
                    'flex-1 text-center font-mono text-[10px]',
                    hover === index ? 'text-foreground' : 'text-muted-foreground',
                  )}
                >
                  {showLabel[index] || hover === index ? bucket.label : ''}
                </span>
              {/each}
            </div>
          </div>

          {#if status === 'all'}
            <div class="flex flex-wrap items-center justify-center gap-4 text-xs">
              {#each SERIES as series (series.key)}
                <span class="flex items-center gap-1.5">
                  <span class="h-2 w-2 rounded-sm" style={`background: ${series.color}`}></span>
                  {seriesLabel(series.key)}
                </span>
              {/each}
            </div>
          {/if}
        {/if}
      </div>
    </CardContent>
  </Card>
{/if}
