<script lang="ts">
  import ActivityIcon from '@lucide/svelte/icons/activity';
  import BanIcon from '@lucide/svelte/icons/ban';
  import CheckCircle2Icon from '@lucide/svelte/icons/check-circle-2';
  import Loader2Icon from '@lucide/svelte/icons/loader-2';
  import UnplugIcon from '@lucide/svelte/icons/unplug';
  import XCircleIcon from '@lucide/svelte/icons/x-circle';
  import { Card, CardContent, CardHeader, CardTitle, Skeleton } from '@shadcn-svelte';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { getRelativeTime } from '@shared/utils/date-utils.ts';
  import type { JobsSummary } from '@/modules/features/jobs';
  import { dashboardMessages } from './dashboard.messages.ts';

  interface Props {
    summary: JobsSummary;
    /** Runs the organization has in total, which may exceed the summarized window. */
    totalRuns: number;
    /** True when `summary` covers only the most recent page of runs. */
    truncated: boolean;
    loading: boolean;
  }

  let { summary, totalRuns, truncated, loading }: Props = $props();

  const inFlight = $derived(summary.pending + summary.running);

  const outcomes = $derived([
    { key: 'completed', icon: CheckCircle2Icon, class: 'text-[var(--success)]', label: t(dashboardMessages.completed), value: summary.completed },
    { key: 'failed', icon: XCircleIcon, class: 'text-destructive', label: t(dashboardMessages.failed), value: summary.failed },
    { key: 'cancelled', icon: BanIcon, class: 'text-muted-foreground', label: t(dashboardMessages.cancelled), value: summary.cancelled },
    { key: 'orphaned', icon: UnplugIcon, class: 'text-amber-500', label: t(dashboardMessages.orphaned), value: summary.orphaned },
  ]);
</script>

<!--
  Organization-wide run activity, from `ListOrganizationJobs`.

  The backend exposes no organization-level aggregate — only a paginated job
  listing — so every figure here is computed over the most recent window. When
  that window does not cover the whole history the card says so rather than
  presenting a partial success rate as if it were the all-time one.
-->
<Card class="py-5">
  <CardHeader class="px-5 pb-0">
    <CardTitle class="flex items-center justify-between text-base font-semibold">
      <span class="flex items-center gap-2">
        <ActivityIcon class="h-4 w-4 text-primary" />
        {t(dashboardMessages.runActivity)}
      </span>
      {#if !loading && inFlight > 0}
        <span class="flex items-center gap-1.5 text-xs font-normal text-muted-foreground">
          <Loader2Icon class="h-3 w-3 animate-spin text-primary" />
          {t(dashboardMessages.inProgress(inFlight))}
        </span>
      {/if}
    </CardTitle>
  </CardHeader>

  <CardContent class="px-5 pb-0 pt-4">
    {#if loading}
      <div class="grid grid-cols-2 gap-4 sm:grid-cols-4">
        {#each { length: 4 } as _, index (index)}
          <Skeleton class="h-10 w-full" />
        {/each}
      </div>
    {:else if summary.total === 0}
      <p class="text-sm text-muted-foreground">{t(dashboardMessages.noRunYet)}</p>
    {:else}
      <div class="grid grid-cols-2 gap-4 sm:grid-cols-4">
        {#each outcomes as outcome (outcome.key)}
          {@const Icon = outcome.icon}
          <div class="flex items-center gap-2">
            <div class="shrink-0"><Icon class="h-4 w-4 {outcome.class}" /></div>
            <div class="min-w-0">
              <p class="text-lg font-semibold leading-none tabular-nums">{outcome.value}</p>
              <p class="truncate text-xs text-muted-foreground mt-0.5">{outcome.label}</p>
            </div>
          </div>
        {/each}
      </div>

      <p class="mt-4 text-xs text-muted-foreground">
        {#if summary.lastRunAt}
          {t(dashboardMessages.lastRun(getRelativeTime(summary.lastRunAt)))} ·
        {/if}
        {truncated
          ? t(dashboardMessages.overWindow(summary.total, totalRuns))
          : t(dashboardMessages.overAll(summary.total))}
      </p>
    {/if}
  </CardContent>
</Card>
