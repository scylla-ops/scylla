<script lang="ts">
  import { Badge } from '@shadcn';
  import { CopyableText } from '@shared/presentation/ui';
  import { createNow } from '@shared/presentation/state/now.svelte.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import {
    calculateExecutionDuration,
    formatDate,
    formatDuration,
  } from '@shared/utils/date-utils.ts';
  import type { JobEntity } from '../../../domain/entities/job.entity.ts';
  import { isActiveStatus } from '../../../domain/structs/jobs-summary.struct.ts';
  import JobStatus from '../jobs-table/JobStatus/JobStatus.svelte';
  import JobTimeline from '../jobs-table/JobTimeline/JobTimeline.svelte';
  import { jobsMessages } from '../jobs.messages.ts';

  interface Props {
    job: JobEntity;
    /**
     * Shows that node's logs alone, closing whatever else was open — picking a
     * segment out of the timeline means reading that one, not adding to a pile.
     * A grouped segment stands for several nodes at once and names none, which
     * comes back to the job as a whole.
     */
    onSelectNode: (nodeId?: string) => void;
  }

  let { job, onSelectNode }: Props = $props();

  // Ticks only while the job is still moving, and stops on its own when it
  // finishes — the getter is what makes that work without an extra effect.
  const now = createNow(() => isActiveStatus(job.status));

  const duration = $derived.by(() => {
    void now.value;
    return calculateExecutionDuration(job.startedAt, job.finishedAt);
  });
</script>

{#snippet fact(label: string, value: string)}
  <span class="flex items-center gap-1.5">
    <span class="font-mono text-[10px] uppercase tracking-wide text-muted-foreground">{label}</span>
    <span class="font-mono text-xs text-foreground">{value}</span>
  </span>
{/snippet}

<div class="flex shrink-0 flex-col gap-4">
  <div class="flex flex-wrap items-center justify-between gap-3">
    <div class="flex min-w-0 items-center gap-3">
      <h1 class="text-xl font-semibold text-foreground">{t(jobsMessages.job)}</h1>
      <Badge variant="outline" class="gap-1 pr-1 font-mono text-xs">
        <CopyableText value={job.id} copyLabel={t(jobsMessages.copyJobId)} />
      </Badge>
    </div>
    <div class="w-fit">
      <JobStatus {job} />
    </div>
  </div>

  <div
    class="flex flex-wrap items-center gap-x-3 gap-y-1.5 rounded-md border border-border bg-card px-3.5 py-2"
  >
    {@render fact(
      t(jobsMessages.duration),
      duration === null ? '-' : formatDuration(duration),
    )}
    <span class="text-muted-foreground/50">·</span>
    {@render fact(
      t(jobsMessages.startedPrefix),
      job.startedAt ? formatDate(job.startedAt) : '-',
    )}
    <span class="text-muted-foreground/50">·</span>
    {@render fact(
      t(jobsMessages.finishedPrefix),
      job.finishedAt ? formatDate(job.finishedAt) : '-',
    )}
    <span class="text-muted-foreground/50">·</span>
    {@render fact(t(jobsMessages.createdPrefix), formatDate(job.createdAt))}
  </div>

  <div>
    <h2 class="mb-2 text-lg font-semibold text-foreground">{t(jobsMessages.timeline)}</h2>
    <JobTimeline nodeExecutions={job.nodeExecutions} {onSelectNode} />
  </div>
</div>
