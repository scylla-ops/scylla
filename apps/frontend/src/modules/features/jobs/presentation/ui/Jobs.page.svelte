<script lang="ts">
  import { loadNoAgentsBanner } from '@/modules/features/agents';
  import { createQuery } from '@platform/query';
  import { ErrorState, PaginationSlot } from '@shared/presentation/ui';
  import { createPagination } from '@shared/presentation/state/pagination.svelte.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { ScyllaError } from '@shared/utils/scylla-result.ts';
  import { jobQueries } from '../jobs.queries.ts';
  import JobsHeader from './JobsHeader/JobsHeader.svelte';
  import JobsTable from './jobs-table/JobsTable.svelte';
  import { jobsMessages } from './jobs.messages.ts';

  interface Props {
    /** Passed by `PipelineJobsRoute`: `pipeline` owns this route (the Run action). */
    pipelineId?: string;
    onRun?: () => Promise<void>;
  }

  let { pipelineId, onRun }: Props = $props();

  const pagination = createPagination({ responsive: true });

  const jobsQuery = createQuery(() =>
    jobQueries.byPipeline(pipelineId ?? '', pagination.paginationParams, {
      enabled: pagination.isPageSizeReady,
    }),
  );

  const jobs = $derived(jobsQuery.data?.items);

  // Clamps the page when the last row of the last page is deleted.
  $effect(() => {
    pagination.updatePaginationInfo(jobsQuery.data?.pagination);
  });

  const errorMessage = $derived(
    jobsQuery.error instanceof ScyllaError
      ? jobsQuery.error.userMessage()
      : t(jobsMessages.loadError),
  );
</script>

{#if !pipelineId}
  <ErrorState message={t(jobsMessages.pipelineIdMissing)} />
{:else if jobsQuery.isError}
  <ErrorState message={errorMessage} />
{:else}
  <!-- The frame renders first: the table area's height decides what to fetch. -->
  <div class="flex flex-col gap-4 w-full h-full min-h-0">
    <JobsHeader
      numberOfJobs={pagination.paginationInfo?.totalCount ?? jobs?.length ?? 0}
      jobIds={jobs?.map(job => job.id) ?? []}
      {pipelineId}
      onRefresh={() => void jobsQuery.refetch()}
      {onRun}
    />

    <!-- A loader from the barrel. Nothing shows until it loads: the banner is advisory. -->
    {#await loadNoAgentsBanner() then banner}
      <banner.default hasPendingJobs={jobs?.some(job => job.status === 'pending') ?? false} />
    {/await}

    <div use:pagination.measure class="flex-1 min-h-0 overflow-auto">
      <div class="relative">
        {#if jobs && jobs.length > 0}
          <JobsTable {jobs} {pipelineId} />
        {:else if jobs}
          <div class="flex items-center justify-center h-full min-h-[400px]">
            <div class="text-center space-y-2">
              <p class="text-muted-foreground">{t(jobsMessages.noJobsFound)}</p>
              <p class="text-sm text-muted-foreground">{t(jobsMessages.noJobsBody)}</p>
            </div>
          </div>
        {/if}
      </div>
    </div>

    <PaginationSlot
      paginationInfo={pagination.paginationInfo}
      onPageChange={page => pagination.setPage(page)}
    />
  </div>
{/if}
