<script lang="ts">
  import type { JobEntity } from '@/modules/features/jobs';
  import { scyllaNavigate } from '@platform/context';
  import { createMutation } from '@platform/query';
  import { DataTable } from '@shared/presentation/ui';
  import { createSelection } from '@shared/presentation/state/selection.svelte.ts';
  import { toStatusState } from '@shared/utils/job-status.utils.ts';
  import type { PipelineMetadata } from '../../../../domain/structs/pipeline.struct.ts';
  import { pipelineMutations } from '../../../pipeline.queries.ts';
  import type { RunPipeline } from '../../../run-pipeline.svelte.ts';
  import PipelineChart from '../PipelineChart.svelte';
  import PipelineActions from './PipelineActions.svelte';
  import PipelineLastJob from './PipelineLastJob.svelte';
  import PipelineStatus from './PipelineStatus.svelte';
  import { pipelineColumns } from './pipeline-columns.ts';

  interface Props {
    pipelines: PipelineMetadata[];
    jobsByPipelineId: Map<string, JobEntity[]>;
    isJobsLoading?: boolean;
    isJobsError?: boolean;
    canListJobs?: boolean;
    /**
     * Shared with the page header, which runs from the same place — one
     * subscription to the agent list between them, and one set of in-flight ids.
     */
    runPipeline: RunPipeline;
  }

  let {
    pipelines,
    jobsByPipelineId,
    isJobsLoading = false,
    isJobsError = false,
    canListJobs = true,
    runPipeline,
  }: Props = $props();

  const selection = createSelection('pipelines');
  const duplicatePipeline = createMutation(() => pipelineMutations.duplicate());

  const jobsOf = (pipelineId: string) => jobsByPipelineId.get(pipelineId) ?? [];

  const columns = $derived(
    pipelineColumns({
      status: statusCell,
      history: historyCell,
      lastRun: lastRunCell,
      actions: actionsCell,
    }),
  );
</script>

{#snippet statusCell(pipeline: PipelineMetadata)}
  <PipelineStatus {pipeline} status={toStatusState(jobsOf(pipeline.id)[0]?.status)} />
{/snippet}

{#snippet historyCell(pipeline: PipelineMetadata)}
  <PipelineChart
    jobs={jobsOf(pipeline.id)}
    isLoading={isJobsLoading}
    isError={isJobsError}
    isForbidden={canListJobs === false}
    maxJobs={10}
    onSelectJob={jobId =>
      scyllaNavigate.goToJobDetails(pipeline.id, jobId, { pipelineName: pipeline.name })}
  />
{/snippet}

{#snippet lastRunCell(pipeline: PipelineMetadata)}
  <PipelineLastJob
    jobs={jobsOf(pipeline.id)}
    onSelectJob={jobId =>
      scyllaNavigate.goToJobDetails(pipeline.id, jobId, { pipelineName: pipeline.name })}
  />
{/snippet}

{#snippet actionsCell(pipeline: PipelineMetadata)}
  <PipelineActions
    onRun={event => {
      // Every action stops the click here: the row itself is a selection
      // target, and acting on a pipeline is not selecting it.
      event.stopPropagation();
      void runPipeline.run(pipeline.id);
    }}
    onEdit={event => {
      event.stopPropagation();
      scyllaNavigate.goToEditPipeline(pipeline.id, pipeline.name);
    }}
    onDuplicate={event => {
      event.stopPropagation();
      duplicatePipeline.mutate(pipeline.id);
    }}
    onViewJobs={event => {
      event.stopPropagation();
      scyllaNavigate.goToJobs(pipeline.id, pipeline.name);
    }}
    onViewTriggers={event => {
      event.stopPropagation();
      scyllaNavigate.goToTriggers(pipeline.id, pipeline.name);
    }}
    isRunning={runPipeline.isRunning(pipeline.id)}
    isDuplicating={duplicatePipeline.isPending && duplicatePipeline.variables === pipeline.id}
  />
{/snippet}

<DataTable
  {columns}
  data={pipelines}
  onRowClick={row => selection.select(row.original.id)}
  getRowId={(pipeline, index) => pipeline.id || String(index)}
  isRowSelected={pipeline => selection.selectedIds.includes(pipeline.id)}
  alignColumnsCenter
  alignRowsCenter
/>
