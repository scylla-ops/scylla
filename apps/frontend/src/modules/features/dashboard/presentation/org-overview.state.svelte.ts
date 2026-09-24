import { Permission, can } from '@platform/authz';
import { contextStore } from '@platform/context';
import { createQuery } from '@platform/query';
import { toRune } from '@shared/presentation/stores/to-rune.svelte.ts';
import { projectQueries } from '@/modules/features/project';
import { asPipelineFeed, pipelineQueries, type PipelineMetadata } from '@/modules/features/pipeline';
import { asJobFeed, jobQueries } from '@/modules/features/jobs';

/** A pipeline carrying the name of the project it belongs to. */
export type PipelineWithProject = PipelineMetadata & { projectName: string };

/** Project ids the user may open — the project route needs this same permission. */
export type ProjectAccess = (projectId: string) => boolean;

/**
 * The organization-wide overview behind the dashboard: the projects the user
 * can see, every pipeline in the organization, and the recent run activity.
 *
 * Each half comes from the module that owns it, through its public API — the
 * dashboard composes, it does not query. That is also why this module has no
 * repository and an empty `domain`: adding one would fork the cache into two
 * keys for one resource.
 *
 * All three calls are organization-scoped and filtered server-side, so there is
 * no client-side permission gate on the data itself. That replaced a
 * per-project fan-out which cost one request per project and produced one
 * `PERMISSION_DENIED` toast for every project the caller could not read.
 * {@link ProjectAccess} remains, for a different question: whether a row the
 * user may *see* leads somewhere they may *enter*.
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
    /** More pipelines exist than the page fetched — counts are a floor. */
    get pipelinesTruncated() {
      return pipelineFeed.isPartialWindow;
    },
    /** Outcome mix over the recent-runs window. */
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
    /** The summary covers a window, not the whole history — say so in the UI. */
    get runsTruncated() {
      return jobFeed.isPartialWindow;
    },
    /**
     * Whether opening this project would land on something the user may see.
     *
     * A method rather than a precomputed set: `can` is reactive, so the answer
     * changes on its own once `usePermissionSync` fills the store, and a row
     * that was inert on the first paint becomes clickable without an effect.
     */
    canOpenProject: ((projectId: string) =>
      can(Permission.LIST_PIPELINES_BY_PROJECT, { projectId })) satisfies ProjectAccess,
  };
};

export type OrgOverview = ReturnType<typeof createOrgOverview>;
