import { useContextStore } from '@platform/context';
import { Permission, useAuthorization } from '@platform/authz';
import { useOrganizationProjects } from '@/modules/features/project';
import { useOrganizationPipelines, type PipelineMetadata } from '@/modules/features/pipeline';
import { useOrganizationJobs, type JobsSummary } from '@/modules/features/jobs';

/** A pipeline carrying the name of the project it belongs to. */
export type PipelineWithProject = PipelineMetadata & { projectName: string };

/** Project ids the user may open — the project route needs this same permission. */
export type ProjectAccess = (projectId: string) => boolean;

/**
 * The organization-wide overview behind the dashboard: the projects the user
 * can see, every pipeline in the organization, and the recent run activity.
 *
 * Each half comes from the module that owns it, through its public API — the
 * dashboard composes, it does not query.
 *
 * All three calls are organization-scoped and filtered server-side, so there is
 * no client-side permission gate on the data itself. That replaced a
 * per-project fan-out which cost one request per project and produced one
 * `PERMISSION_DENIED` toast for every project the caller could not read.
 * `canOpenProject` remains, for a different question: whether a row the user
 * may *see* leads somewhere they may *enter*.
 */
export const useOrgOverview = () => {
  const organizationId = useContextStore(state => state.organization.id);
  const { can } = useAuthorization();

  const {
    projects,
    isLoading: projectsLoading,
    isError: projectsError,
  } = useOrganizationProjects(organizationId);

  const {
    pipelines,
    isLoading: pipelinesLoading,
    isPartialWindow: pipelinesTruncated,
  } = useOrganizationPipelines(organizationId);

  const {
    jobs: recentJobs,
    summary: runs,
    totalCount: totalRuns,
    isLoading: runsLoading,
    isPartialWindow: runsTruncated,
  } = useOrganizationJobs(organizationId);

  // The organization listing carries `projectId` but not the project's name,
  // so the label is joined here rather than asked of the server again.
  const projectNameById = new Map(projects.map(project => [project.id, project.name]));

  const allPipelines: PipelineWithProject[] = pipelines.map(pipeline => ({
    ...pipeline,
    projectName: projectNameById.get(pipeline.projectId) ?? '',
  }));

  return {
    projects,
    projectsLoading,
    projectsError,
    allPipelines,
    pipelinesLoading,
    /** More pipelines exist than the page fetched — counts are a floor. */
    pipelinesTruncated,
    /** Outcome mix over the recent-runs window. */
    runs: runs satisfies JobsSummary,
    recentJobs,
    totalRuns,
    runsLoading,
    /** The summary covers a window, not the whole history — say so in the UI. */
    runsTruncated,
    organizationId,
    /** Whether opening this project would land on something the user may see. */
    canOpenProject: ((projectId: string) =>
      can(Permission.LIST_PIPELINES_BY_PROJECT, { projectId })) satisfies ProjectAccess,
  };
};
