<script lang="ts">
  import { ErrorState, PaginationSlot } from '@shared/presentation/ui';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { createPipelineDashboard } from '../../pipeline-dashboard.state.svelte.ts';
  import { createRunPipeline } from '../../run-pipeline.svelte.ts';
  import { pipelineMessages } from '../../pipeline.messages.ts';
  import PipelineDashboardHeader from './PipelineDashboardHeader.svelte';
  import PipelineTable from './pipeline-table/PipelineTable.svelte';

  interface Props {
    /** From the route: this page is a project's index. */
    projectId?: string;
  }

  let { projectId }: Props = $props();

  const dashboard = createPipelineDashboard(() => projectId ?? '');
  const runPipeline = createRunPipeline();
</script>

{#if dashboard.isError}
  <ErrorState message={dashboard.errorMessage} />
{:else}
  <!-- The frame renders before the pipelines do: the table area has to be in the
       DOM for its height to be measured, and that height decides what to fetch. -->
  <div class="flex h-full min-h-0 w-full flex-col gap-4">
    <PipelineDashboardHeader
      numberOfPipelines={dashboard.totalCount}
      pipelineIds={dashboard.pipelineIds}
    />

    <div use:dashboard.pagination.measure class="min-h-0 flex-1 overflow-auto">
      <div class="relative">
        {#if dashboard.pipelines && dashboard.pipelines.length > 0}
          <PipelineTable
            pipelines={dashboard.pipelines}
            jobsByPipelineId={dashboard.jobsByPipelineId}
            isJobsLoading={dashboard.isJobsLoading}
            isJobsError={dashboard.isJobsError}
            canListJobs={dashboard.canListJobs}
            {runPipeline}
          />
        {:else if dashboard.pipelines}
          <div class="flex h-full min-h-100 items-center justify-center">
            <div class="space-y-2 text-center">
              <p class="text-muted-foreground">{t(pipelineMessages.noPipelines)}</p>
              <p class="text-sm text-muted-foreground">{t(pipelineMessages.noPipelinesBody)}</p>
            </div>
          </div>
        {/if}
      </div>
    </div>

    <PaginationSlot
      paginationInfo={dashboard.pagination.paginationInfo}
      onPageChange={page => dashboard.pagination.setPage(page)}
    />
  </div>
{/if}
