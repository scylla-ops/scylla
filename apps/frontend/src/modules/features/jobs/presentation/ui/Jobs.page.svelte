<script lang="ts">
  import { loadNoAgentsBanner } from '@/modules/features/agents';
  import { createQuery } from '@platform/query';
  import { ErrorState, PaginationSlot } from '@shared/presentation/ui-svelte';
  import { createPagination } from '@shared/presentation/state/pagination.svelte.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { ScyllaError } from '@shared/utils/scylla-result.ts';
  import { jobQueries } from '../jobs.queries.ts';
  import JobsHeader from './JobsHeader.svelte';
  import JobsTable from './jobs-table/JobsTable.svelte';
  import { jobsMessages } from './jobs.messages.ts';

  interface Props {
    /**
     * From the route this page is mounted under. `pipeline` owns it — the list
     * needs a Run action, which is a pipeline operation — so it arrives as a
     * prop from `PipelineJobsRoute` rather than from `sveltePage`.
     */
    pipelineId?: string;
    /** Runs the pipeline. See {@link JobsHeader}'s note on the same prop. */
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

  // The clamp inside `updatePaginationInfo` is remembered state — the page the
  // reader is on can stop existing when the last row of the last page is
  // deleted — so this is synchronisation with the server's answer, not a mirror
  // of it. The React hook needed the same effect for the same reason.
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
  <!-- The frame renders before the jobs do: the table area has to be in the DOM
       for its height to be measured, and that height decides what to fetch. -->
  <div class="flex flex-col gap-4 w-full h-full min-h-0">
    <JobsHeader
      numberOfJobs={pagination.paginationInfo?.totalCount ?? jobs?.length ?? 0}
      jobIds={jobs?.map(job => job.id) ?? []}
      {pipelineId}
      onRefresh={() => void jobsQuery.refetch()}
      {onRun}
    />

    <!-- Awaited, not imported: the barrel hands out a loader so bits-ui stays
         out of the chunks of its React consumers. Nothing is shown until it
         lands, which is right — the banner is advisory. -->
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
