<script lang="ts">
  import type { Component } from 'svelte';
  import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
  import FolderIcon from '@lucide/svelte/icons/folder';
  import GaugeIcon from '@lucide/svelte/icons/gauge';
  import PlayCircleIcon from '@lucide/svelte/icons/play-circle';
  import WorkflowIcon from '@lucide/svelte/icons/workflow';
  import { can, Permission } from '@platform/authz';
  import { scyllaNavigate } from '@platform/context';
  import {
    Badge,
    Card,
    CardContent,
    CardHeader,
    CardTitle,
    Separator,
    Skeleton,
  } from '@shadcn-svelte';
  import { ErrorState, FeatureHeader } from '@shared/presentation/ui-svelte';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { getRelativeTime } from '@shared/utils/date-utils.ts';
  import { createOrgOverview } from '../org-overview.state.svelte.ts';
  import AgentOutcomesChart from './AgentOutcomesChart.svelte';
  import RunActivityCard from './RunActivityCard.svelte';
  import { dashboardMessages } from './dashboard.messages.ts';

  const overview = createOrgOverview();

  const stats = $derived([
    {
      key: 'projects',
      icon: FolderIcon as Component,
      label: t(dashboardMessages.projects),
      value: String(overview.projects.length),
      loading: overview.projectsLoading,
    },
    {
      key: 'pipelines',
      icon: WorkflowIcon as Component,
      label: t(dashboardMessages.pipelines),
      value: String(overview.allPipelines.length),
      loading: overview.pipelinesLoading || overview.projectsLoading,
    },
    {
      key: 'runs',
      icon: PlayCircleIcon as Component,
      label: t(dashboardMessages.runs),
      value: String(overview.totalRuns),
      loading: overview.runsLoading,
    },
    {
      key: 'success',
      icon: GaugeIcon as Component,
      label: overview.runsTruncated
        ? t(dashboardMessages.successRateRecent)
        : t(dashboardMessages.successRate),
      // `null` means nothing has finished yet — not a 0 % success rate.
      value:
        overview.runs.successRate === null
          ? '—'
          : `${Math.round(overview.runs.successRate * 100)}%`,
      loading: overview.runsLoading,
    },
  ]);

  const sortedPipelines = $derived(
    [...overview.allPipelines].sort(
      (a, b) => a.projectName.localeCompare(b.projectName) || a.name.localeCompare(b.name),
    ),
  );

  /**
   * A project the user may see listed but not open: the row stays inert rather
   * than sending them to a permission-denied panel.
   */
  const openPipelineRow = (projectId: string) => {
    const project = overview.projects.find(candidate => candidate.id === projectId);
    if (project && overview.canOpenProject(project.id)) {
      scyllaNavigate.goToProject(project.id, project.name);
    }
  };
</script>

<!--
  The organization landing page.

  A fixed frame: header, stats and chart keep their own height (`shrink-0`) and
  the two lists share what is left, each scrolling inside its own box. The lists
  carry a `min-h` floor so they stay usable instead of being squeezed to a
  sliver; if a viewport is too short to honour even that, the frame itself
  scrolls rather than collapsing them out of sight. `pr-2` keeps right-aligned
  content off the scroll track, unconditionally, so the layout does not shift
  depending on whether a scrollbar happens to be showing.
-->
{#if overview.projectsError}
  <ErrorState message={t(dashboardMessages.loadError)} />
{:else}
  <div class="flex h-full min-h-0 w-full flex-col gap-4">
    <FeatureHeader label={t(dashboardMessages.dashboard)} />

    <div class="grid shrink-0 grid-cols-2 gap-4 md:grid-cols-4">
      {#each stats as stat (stat.key)}
        {@const Icon = stat.icon}
        <Card class="py-5">
          <CardContent class="px-5 pb-0">
            <div class="flex items-center gap-3">
              <div class="rounded-lg bg-primary/10 p-2 shrink-0">
                <Icon class="h-4 w-4 text-primary" />
              </div>
              <div>
                {#if stat.loading}
                  <Skeleton class="h-7 w-10 mb-1" />
                {:else}
                  <p class="text-2xl font-bold leading-none">{stat.value}</p>
                {/if}
                <p class="text-xs text-muted-foreground mt-1">{stat.label}</p>
              </div>
            </div>
          </CardContent>
        </Card>
      {/each}
    </div>

    <div class="shrink-0">
      <RunActivityCard
        summary={overview.runs}
        totalRuns={overview.totalRuns}
        truncated={overview.runsTruncated}
        loading={overview.runsLoading}
      />
    </div>

    <Separator class="shrink-0" />

    <section class="flex flex-1 flex-col gap-3">
      <div class="flex shrink-0 items-center justify-between">
        <h2 class="text-base font-semibold">{t(dashboardMessages.projects)}</h2>
        <button
          type="button"
          onclick={() => scyllaNavigate.goToOrgRoute('/projects')}
          class="flex items-center gap-1 text-sm text-muted-foreground transition-colors hover:text-foreground"
        >
          {t(dashboardMessages.seeAll)}
          <ChevronRightIcon class="h-3.5 w-3.5" />
        </button>
      </div>

      <div class="min-h-[8rem] flex-1 overflow-y-auto pr-2">
        {#if overview.projectsLoading}
          <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3">
            {#each { length: 4 } as _, index (index)}
              <Skeleton class="h-20 w-full rounded-xl" />
            {/each}
          </div>
        {:else if overview.projects.length === 0}
          <p class="text-sm text-muted-foreground">{t(dashboardMessages.noProjects)}</p>
        {:else}
          <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3">
            {#each overview.projects as project (project.id)}
              {@const openable = overview.canOpenProject(project.id)}
              <Card
                onclick={openable
                  ? () => scyllaNavigate.goToProject(project.id, project.name)
                  : undefined}
                title={openable ? undefined : t(dashboardMessages.noProjectAccess)}
                class={cn(
                  'py-4 gap-2 transition-all',
                  openable
                    ? 'cursor-pointer hover:shadow-md hover:border-primary/50 active:scale-[0.98]'
                    : 'opacity-60',
                )}
              >
                <CardHeader class="px-4 pb-0">
                  <div class="flex items-center gap-2">
                    <div class="rounded-md bg-primary/10 p-1.5 shrink-0">
                      <FolderIcon class="h-3.5 w-3.5 text-primary" />
                    </div>
                    <CardTitle class="text-sm font-semibold truncate" title={project.name}>
                      {project.name}
                    </CardTitle>
                  </div>
                </CardHeader>
                <CardContent class="px-4 pb-0">
                  <p class="text-xs text-muted-foreground truncate">
                    {#if project.description}
                      {project.description}
                    {:else}
                      <span class="italic">{t(dashboardMessages.noDescription)}</span>
                    {/if}
                  </p>
                </CardContent>
              </Card>
            {/each}
          </div>
        {/if}
      </div>
    </section>

    <Separator class="shrink-0" />

    <section class="flex flex-1 flex-col gap-3">
      <h2 class="shrink-0 text-base font-semibold">{t(dashboardMessages.allPipelines)}</h2>

      <div class="min-h-[10rem] flex-1 overflow-y-auto pr-2">
        {#if overview.projectsLoading || overview.pipelinesLoading}
          <div class="flex flex-col gap-2">
            {#each { length: 5 } as _, index (index)}
              <Skeleton class="h-12 w-full rounded-xl" />
            {/each}
          </div>
        {:else if sortedPipelines.length === 0}
          <p class="text-sm text-muted-foreground">{t(dashboardMessages.noPipelines)}</p>
        {:else}
          <div class="rounded-xl border overflow-hidden">
            <table class="w-full text-sm">
              <!--
                Sticky inside the section's scroll box — needs an opaque
                background so rows don't show through as they pass under.
              -->
              <thead class="sticky top-0 z-10 border-b bg-muted">
                <tr>
                  <th class="px-4 py-3 text-left font-medium text-muted-foreground">
                    {t(dashboardMessages.name)}
                  </th>
                  <th
                    class="hidden px-4 py-3 text-left font-medium text-muted-foreground sm:table-cell"
                  >
                    {t(dashboardMessages.project)}
                  </th>
                  <th
                    class="hidden px-4 py-3 text-left font-medium text-muted-foreground md:table-cell"
                  >
                    {t(dashboardMessages.nodes)}
                  </th>
                  <th
                    class="hidden px-4 py-3 text-left font-medium text-muted-foreground md:table-cell"
                  >
                    {t(dashboardMessages.updated)}
                  </th>
                </tr>
              </thead>
              <tbody>
                {#each sortedPipelines as pipeline, index (pipeline.id)}
                  {@const openable = overview.canOpenProject(pipeline.projectId)}
                  <tr
                    onclick={() => openPipelineRow(pipeline.projectId)}
                    class={cn(
                      'transition-colors hover:bg-muted/40',
                      openable && 'cursor-pointer',
                      index > 0 && 'border-t',
                    )}
                  >
                    <td class="px-4 py-3 font-medium">{pipeline.name}</td>
                    <td class="hidden px-4 py-3 sm:table-cell">
                      <Badge variant="secondary">{pipeline.projectName}</Badge>
                    </td>
                    <td class="hidden px-4 py-3 text-muted-foreground md:table-cell">
                      {pipeline.nodeCount}
                    </td>
                    <td class="hidden px-4 py-3 text-muted-foreground md:table-cell">
                      {getRelativeTime(pipeline.updatedAt)}
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>
    </section>

    <!--
      The chart reads agent stats — hidden whole rather than rendering a panel
      that can only fail for a user without agent access.
    -->
    {#if can(Permission.READ_APP_STATS)}
      <Separator class="shrink-0" />

      <section class="flex shrink-0 flex-col gap-3">
        <h2 class="text-base font-semibold">{t(dashboardMessages.agentOutcomes)}</h2>
        <AgentOutcomesChart />
      </section>
    {/if}
  </div>
{/if}
