<script lang="ts">
  import { createResourceError } from '@platform/context';
  import { createQuery } from '@platform/query';
  import { Skeleton } from '@shadcn';
  import { ErrorState } from '@shared/presentation/ui';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { jobQueries } from '../jobs.queries.ts';
  import { createOpenLogPanels } from '../open-log-panels.svelte.ts';
  import JobNodeLogs from './job-details/JobNodeLogs.svelte';
  import JobSummary from './job-details/JobSummary.svelte';
  import { nodeIdOf } from './jobs-table/job-timeline.calculator.ts';
  import { jobsMessages } from './jobs.messages.ts';

  interface Props {
    /** From the route: `…/pipelines/:pipelineId/jobs/:jobId`. */
    jobId?: string;
  }

  let { jobId }: Props = $props();

  const jobQuery = createQuery(() => jobQueries.byId(jobId ?? ''));
  const job = $derived(jobQuery.data);

  // A getter: the executions arrive with the job, and a list read once would
  // filter the URL against an empty set on the first paint, dropping every
  // panel the incoming link asked for.
  const panels = createOpenLogPanels(() => job?.nodeExecutions.map(nodeIdOf) ?? []);

  const resourceError = createResourceError({
    error: () => jobQuery.error,
    redirectTo: '..',
    notFoundMessage: t(jobsMessages.jobNotFound),
  });
</script>

<!--
  One job: what it did, and what it printed.

  Which log panels are open lives in the URL rather than in state, so a link can
  open the page already showing one node's logs — which is what the timeline
  segments on the jobs list and the pipeline dashboard link to, and what this
  page's own timeline does to the page it is already on.

  The page fills the viewport instead of growing with its content: the logs are
  what the page is for, so they take the room the summary leaves and scroll
  inside it, rather than pushing the whole page into a scroll of its own.
-->
{#if !jobId}
  <ErrorState message={t(jobsMessages.jobIdMissing)} />
{:else if resourceError.redirecting}
  <!-- Nothing: the redirect is already in flight. -->
{:else if jobQuery.isLoading}
  <Skeleton class="h-72 w-full rounded-xl" />
{:else if jobQuery.isError || !job}
  <ErrorState message={t(jobsMessages.jobLoadError)} />
{:else}
  <div class="flex h-full min-h-0 w-full flex-col gap-6">
    <JobSummary {job} onSelectNode={panels.selectNode} />
    <JobNodeLogs
      {job}
      openNodeIds={panels.openNodeIds}
      isWholeJobOpen={panels.isWholeJobOpen}
      onToggleNode={panels.toggleNode}
      onShowWholeJob={() => panels.selectNode()}
    />
  </div>
{/if}
