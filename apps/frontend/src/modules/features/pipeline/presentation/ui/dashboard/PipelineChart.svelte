<script lang="ts">
  import type { JobEntity } from '@/modules/features/jobs';
  import { Skeleton } from '@shadcn';
  import { StatusBar, type StatusBarItem } from '@shared/presentation/ui';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { calculateDuration, formatDuration, getRelativeTime } from '@shared/utils/date-utils.ts';
  import { getStatusConfig } from '@shared/utils/status-config.ts';
  import { pipelineMessages } from '../../pipeline.messages.ts';

  interface Props {
    jobs: JobEntity[];
    maxJobs?: number;
    isLoading?: boolean;
    isError?: boolean;
    /** The job history was never fetched — the user may not list this project's jobs. */
    isForbidden?: boolean;
    /** Makes each run in the history open that job's details page. */
    onSelectJob?: (jobId: string) => void;
  }

  let { jobs, maxJobs, isLoading = false, isError = false, isForbidden = false, onSelectJob }: Props =
    $props();

  /** The bar's item, carrying what its tooltip needs — one snippet serves them all. */
  type RunItem = StatusBarItem & { job: JobEntity; runNumber: number };

  const items = $derived.by((): RunItem[] =>
    jobs
      .slice(0, maxJobs)
      .map((job, index) => ({
        id: job.id,
        status: job.status,
        // Oldest on the left, so run numbers count up the way the eye reads.
        runNumber: jobs.length - index,
        job,
        label: t(pipelineMessages.runNumber(jobs.length - index)),
        onSelect: onSelectJob ? () => onSelectJob(job.id) : undefined,
      }))
      .reverse(),
  );
</script>

<!-- A pipeline's recent runs as a strip of coloured segments, newest last. -->
{#snippet runTooltip(item: RunItem)}
  {@const config = getStatusConfig(item.job.status)}
  {@const duration = calculateDuration(item.job.createdAt, item.job.updatedAt)}
  <div class="flex flex-col gap-1.5">
    <div class="flex items-center justify-between gap-4">
      <span class="font-bold text-muted-foreground">{t(pipelineMessages.runNumber(item.runNumber))}</span>
      <span class="font-mono text-[10px] text-muted-foreground">{item.job.id.slice(0, 8)}...</span>
    </div>
    <div class="flex items-center gap-2">
      <div class={cn('h-2 w-2 rounded-full', config.dotClassName)}></div>
      <span class={cn('font-semibold', config.textClassName)}>{t(config.label)}</span>
    </div>
    <span class="mt-1 border-t pt-1 text-[10px] text-muted-foreground italic">
      {item.job.status === 'running' || item.job.status === 'pending'
        ? t(pipelineMessages.startedAt(getRelativeTime(item.job.createdAt)))
        : t(pipelineMessages.finishedAt(getRelativeTime(item.job.updatedAt)))}
      • {t(pipelineMessages.durationOf(formatDuration(duration)))}
    </span>
  </div>
{/snippet}

{#if isLoading}
  <div class="flex h-10 w-full items-center gap-2 overflow-hidden rounded-md px-1 py-1">
    {#each { length: maxJobs ?? 0 } as _, index (index)}
      <Skeleton class="h-full min-w-[4px] flex-1 rounded-sm" />
    {/each}
  </div>
{:else if isForbidden}
  <!-- Checked before the error state: nothing was requested, so there is no
       failure to report — only a permission the user doesn't hold. -->
  <div class="flex h-10 w-full items-center justify-center py-1">
    <span class="text-xs text-muted-foreground italic">{t(pipelineMessages.jobsForbidden)}</span>
  </div>
{:else if isError}
  <div class="flex h-10 w-full items-center justify-center py-1">
    <span class="text-xs text-muted-foreground italic">{t(pipelineMessages.jobsError)}</span>
  </div>
{:else}
  <StatusBar
    {items}
    tooltip={runTooltip}
    emptyLabel={t(pipelineMessages.noJobsYet)}
    height="h-10"
    class="px-1"
  />
{/if}
