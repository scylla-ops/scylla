<script lang="ts">
  import { createQuery } from '@platform/query';
  import { ErrorState, Pagination } from '@shared/presentation/ui';
  import { createPagination } from '@shared/presentation/state/pagination.svelte.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { projectQueries } from '../project.queries.ts';
  import ProjectCard from './ProjectCard/ProjectCard.svelte';
  import ProjectHeader from './ProjectHeader.svelte';
  import { projectMessages } from './project.messages.ts';

  interface Props {
    organizationId: string;
  }

  let { organizationId }: Props = $props();

  const pagination = createPagination();

  const projectsQuery = createQuery(() =>
    projectQueries.byOrganization(organizationId, pagination.paginationParams),
  );

  const projects = $derived(projectsQuery.data?.projects);
  const paginationInfo = $derived(projectsQuery.data?.pagination);
</script>

{#if projectsQuery.isLoading}
  <!-- Nothing while loading: the grid would only flash. -->
{:else if projectsQuery.isError || !projects}
  <ErrorState message={t(projectMessages.loadError)} />
{:else}
  <div class="flex min-h-full w-full flex-col gap-4">
    <ProjectHeader
      numberOfProjects={paginationInfo?.totalCount ?? projects.length}
      projectIds={projects.map(project => project.id)}
    />
    <!-- Breakpoints are viewport-wide and the sidebar takes ~16rem: late 3rd and 4th columns. -->
    <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 2xl:grid-cols-4">
      {#each projects as project (project.id)}
        <ProjectCard {project} />
      {/each}
    </div>
    {#if paginationInfo && paginationInfo.totalPages > 1}
      <Pagination {paginationInfo} onPageChange={page => pagination.setPage(page)} class="pb-2" />
    {/if}
  </div>
{/if}
