<script lang="ts">
  import ClockIcon from '@lucide/svelte/icons/clock';
  import type { JobEntity } from '@/modules/features/jobs';
  import { createNow } from '@shared/presentation/state/now.svelte.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { calculateExecutionDuration, formatDuration, getRelativeTime } from '@shared/utils/date-utils.ts';
  import { pipelineMessages } from '../../../pipeline.messages.ts';

  interface Props {
    jobs: JobEntity[];
    onSelectJob?: (jobId: string) => void;
  }

  let { jobs, onSelectJob }: Props = $props();

  const lastJob = $derived(jobs[0]);
  const isLive = $derived(lastJob?.status === 'running' || lastJob?.status === 'pending');

  // A running job has no end: the clock moves, and stops when it finishes.
  const now = createNow(() => isLive);

  const duration = $derived.by(() => {
    if (!lastJob) return null;
    void now.value;
    return calculateExecutionDuration(lastJob.startedAt, lastJob.finishedAt);
  });

  const className = 'flex flex-col w-full items-center justify-center gap-1';
</script>

{#snippet body()}
  <div class="flex w-full flex-row items-center justify-center gap-1.5">
    <ClockIcon class="h-3.5 w-3.5" />
    <span>{duration === null ? '-' : formatDuration(duration)}</span>
  </div>
  <span class="truncate text-xs italic">{getRelativeTime(lastJob.updatedAt)}</span>
{/snippet}

{#if !lastJob}
  <div class={className}>
    <div class="flex items-center justify-center gap-1.5">
      <ClockIcon class="h-3.5 w-3.5" />
      <span>-</span>
    </div>
    <span class="truncate text-xs italic">{t(pipelineMessages.noJobsYet)}</span>
  </div>
{:else if onSelectJob}
  <button
    type="button"
    aria-label={t(pipelineMessages.openLastRun)}
    onclick={event => {
      event.stopPropagation();
      onSelectJob(lastJob.id);
    }}
    class="{className} cursor-pointer rounded-md px-1 py-0.5 transition-colors hover:bg-accent hover:text-accent-foreground"
  >
    {@render body()}
  </button>
{:else}
  <div class={className}>{@render body()}</div>
{/if}
