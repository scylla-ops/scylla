import { Permission, can } from '@platform/authz';
import { contextStore } from '@platform/context';
import { createQuery } from '@platform/query';
import { toRune } from '@shared/presentation/stores/to-rune.svelte.ts';
import { projectQueries } from '@/modules/features/project';
import { asPipelineFeed, pipelineQueries, type PipelineMetadata } from '@/modules/features/pipeline';
import { asJobFeed, jobQueries } from '@/modules/features/jobs';

export type PipelineWithProject = PipelineMetadata & { projectName: string };

/** Whether the user may open the project (the project route needs the same permission). */
export type ProjectAccess = (projectId: string) => boolean;

/**
 * Projects, pipelines and recent runs of the organization, each from the module
 * that owns it. Scoped by the backend: no permission gate on the data.
 */
export const createOrgOverview = () => {
  const context = toRune(contextStore);
  const organizationId = $derived(context().organization.id);

  const projectsQuery = createQuery(() => projectQueries.lookup(organizationId));
  const pipelinesQuery = createQuery(() => pipelineQueries.byOrganization(organizationId));
  const jobsQuery = createQuery(() => jobQueries.byOrganization(organizationId));

  const projects = $derived(projectsQuery.data?.projects ?? []);
  const pipelineFeed = $derived(asPipelineFeed(pipelinesQuery.data));
  const jobFeed = $derived(asJobFeed(jobsQuery.data));

  // The organization listing carries `projectId` but not the project's name, so
  // the label is joined here rather than asked of the server again.
  // Rebuilt whole by the `$derived` and never mutated after it is read, so a
  // reactive collection would only make a throwaway object track dependencies.
  // eslint-disable-next-line svelte/prefer-svelte-reactivity
  const projectNameById = $derived(new Map(projects.map(project => [project.id, project.name])));

  const allPipelines = $derived.by((): PipelineWithProject[] =>
    pipelineFeed.pipelines.map(pipeline => ({
      ...pipeline,
      projectName: projectNameById.get(pipeline.projectId) ?? '',
    })),
  );

  return {
    get organizationId() {
      return organizationId;
    },
    get projects() {
      return projects;
    },
    get projectsLoading() {
      return projectsQuery.isLoading;
    },
    get projectsError() {
      return projectsQuery.isError;
    },
    get allPipelines() {
      return allPipelines;
    },
    get pipelinesLoading() {
      return pipelinesQuery.isLoading;
    },
    /** Counts are a floor. */
    get pipelinesTruncated() {
      return pipelineFeed.isPartialWindow;
    },
    get runs() {
      return jobFeed.summary;
    },
    get recentJobs() {
      return jobFeed.jobs;
    },
    get totalRuns() {
      return jobFeed.totalCount;
    },
    get runsLoading() {
      return jobsQuery.isLoading;
    },
    /** Say so in the UI. */
    get runsTruncated() {
      return jobFeed.isPartialWindow;
    },
    /** Reactive: a row becomes clickable once the permissions load. */
    canOpenProject: ((projectId: string) =>
      can(Permission.LIST_PIPELINES_BY_PROJECT, { projectId })) satisfies ProjectAccess,
  };
};

export type OrgOverview = ReturnType<typeof createOrgOverview>;
