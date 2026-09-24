<script lang="ts">
  import { scyllaNavigate } from '@platform/context';
  import { createMutation } from '@platform/query';
  import {
    ConfirmOperationAlertDialog,
    CopyableText,
    DataTable,
  } from '@shared/presentation/ui';
  import { createNow } from '@shared/presentation/state/now.svelte.ts';
  import { createSelection } from '@shared/presentation/state/selection.svelte.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import {
    calculateExecutionDuration,
    formatDuration,
    getRelativeTime,
  } from '@shared/utils/date-utils.ts';
  import type { JobEntity } from '../../../domain/entities/job.entity.ts';
  import { isActiveStatus } from '../../../domain/structs/jobs-summary.struct.ts';
  import { jobMutations } from '../../jobs.queries.ts';
  import JobActions from './JobActions/JobActions.svelte';
  import JobStatus from './JobStatus/JobStatus.svelte';
  import JobTimeline from './JobTimeline/JobTimeline.svelte';
  import { jobColumns } from './job-columns.ts';
  import { jobsMessages } from '../jobs.messages.ts';

  interface Props {
    jobs: JobEntity[];
    pipelineId: string;
  }

  let { jobs, pipelineId }: Props = $props();

  const selection = createSelection('jobs');
  const deleteJob = createMutation(() => jobMutations.remove(pipelineId));

  let jobToDelete = $state<string | null>(null);

  // One clock for the table, stopped when nothing runs.
  const now = createNow(() => jobs.some(job => isActiveStatus(job.status)));

  const durationOf = (job: JobEntity) => {
    void now.value;
    return calculateExecutionDuration(job.startedAt, job.finishedAt);
  };

  const columns = $derived(
    jobColumns({
      status: statusCell,
      id: idCell,
      timeline: timelineCell,
      duration: durationCell,
      created: createdCell,
      actions: actionsCell,
    }),
  );
</script>

{#snippet statusCell(job: JobEntity)}
  <JobStatus {job} />
{/snippet}

{#snippet idCell(job: JobEntity)}
  <CopyableText value={job.id} truncate={12} copyLabel={t(jobsMessages.copyId)} />
{/snippet}

{#snippet timelineCell(job: JobEntity)}
  <JobTimeline
    nodeExecutions={job.nodeExecutions}
    onSelectNode={nodeId => scyllaNavigate.goToJobDetails(pipelineId, job.id, { nodeId })}
  />
{/snippet}

{#snippet durationCell(job: JobEntity)}
  {@const duration = durationOf(job)}
  <span class="text-sm font-medium whitespace-nowrap">
    {duration === null ? '-' : formatDuration(duration)}
  </span>
{/snippet}

{#snippet createdCell(job: JobEntity)}
  <span class="text-sm whitespace-nowrap">{getRelativeTime(job.createdAt)}</span>
{/snippet}

{#snippet actionsCell(job: JobEntity)}
  <JobActions
    onView={event => {
      event.stopPropagation();
      scyllaNavigate.goToJobDetails(pipelineId, job.id);
    }}
    onDelete={event => {
      event.stopPropagation();
      jobToDelete = job.id;
    }}
  />
{/snippet}

<DataTable
  {columns}
  data={jobs}
  onRowClick={row => selection.select(row.original.id)}
  getRowId={(job, index) => job.id || String(index)}
  isRowSelected={job => selection.selectedIds.includes(job.id)}
  alignColumnsCenter
  alignRowsCenter
/>

<ConfirmOperationAlertDialog
  open={jobToDelete !== null}
  onOpenChange={open => {
    if (!open) jobToDelete = null;
  }}
  onContinue={() => {
    if (!jobToDelete) return;
    deleteJob.mutate(jobToDelete);
    jobToDelete = null;
  }}
  title={t(jobsMessages.deleteJobTitle)}
  description={t(jobsMessages.deleteJobBody(jobToDelete ?? ''))}
/>
