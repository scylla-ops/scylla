import { currentPathname, navigateBack, navigateTo } from './navigator.ts';
import { useContextStore } from './use-context.store.ts';
import { slugifyOrgName } from '@shared/utils/slug.ts';

/**
 * Context-aware navigation, with no framework in it.
 *
 * Every function reads the current organization / project from the store at
 * call time and builds the URL from it, so callers never hand-assemble
 * `/${orgSlug}/projects/${projectId}/…`. That was already true of the React
 * hook this replaces; what changed is that nothing here is a hook any more, so
 * a Svelte view model imports `scyllaNavigate` directly and
 * `useScyllaNavigate()` is a one-line binding for the React half.
 */

const getOrgPrefix = () => {
  const orgName = useContextStore.getState().organization.name;
  return orgName ? `/${slugifyOrgName(orgName)}` : '';
};

const goToSubRoute = (subPath: string, options = {}) => {
  const pathname = currentPathname();
  const base = pathname.endsWith('/') ? pathname.slice(0, -1) : pathname;
  const cleanSubPath = subPath.startsWith('/') ? subPath.slice(1) : subPath;
  navigateTo(`${base}/${cleanSubPath}`, options);
};

// Navigation takes ids and names, never feature entities: this is shared
// infrastructure and must stay ignorant of what a project or a pipeline is.
const goToProject = (id: string, name: string) => {
  navigateTo(`${getOrgPrefix()}/projects/${id}`);
  useContextStore.getState().setProject(id, name);
};

const goToCreatePipeline = () => {
  navigateTo(`${getOrgPrefix()}/projects/${useContextStore.getState().project.id}/create`);
};

const goToEditPipeline = (id: string, name: string) => {
  navigateTo(`${getOrgPrefix()}/projects/${useContextStore.getState().project.id}/edit/${id}`);
  useContextStore.getState().setPipeline(id, name);
};

// `name` is optional: a caller already inside the pipeline (its jobs, one of
// its jobs) navigates without renaming the context it is already in.
const goToJobs = (id: string, name?: string) => {
  const projectId = useContextStore.getState().project.id;
  navigateTo(`${getOrgPrefix()}/projects/${projectId}/pipelines/${id}/jobs`);
  if (name) useContextStore.getState().setPipeline(id, name);
};

const goToJobDetails = (
  pipelineId: string,
  jobId: string,
  options: { nodeId?: string; pipelineName?: string } = {},
) => {
  const projectId = useContextStore.getState().project.id;
  // `nodes` is a list the page opens a panel per id for; one id opens that
  // node's logs alone, which is what every caller here means.
  const query = options.nodeId ? `?nodes=${encodeURIComponent(options.nodeId)}` : '';
  navigateTo(
    `${getOrgPrefix()}/projects/${projectId}/pipelines/${pipelineId}/jobs/${jobId}${query}`,
  );
  if (options.pipelineName)
    useContextStore.getState().setPipeline(pipelineId, options.pipelineName);
};

const goToTriggers = (id: string, name: string) => {
  const projectId = useContextStore.getState().project.id;
  navigateTo(`${getOrgPrefix()}/projects/${projectId}/pipelines/${id}/triggers`);
  useContextStore.getState().setPipeline(id, name);
};

const goToUserSettings = (userId: string) => {
  navigateTo(`${getOrgPrefix()}/users/${userId}`, { replace: true });
};

const goToAgentDetails = (agentId: string) => {
  navigateTo(`${getOrgPrefix()}/agents/${agentId}`);
};

const goToOrgRoute = (path: string) => {
  navigateTo(`${getOrgPrefix()}${path.startsWith('/') ? path : `/${path}`}`);
};

export const scyllaNavigate = {
  navigate: navigateTo,
  goToEditPipeline,
  goToUserSettings,
  goToSubRoute,
  goToCreatePipeline,
  goToJobs,
  goToJobDetails,
  goToTriggers,
  goToAgentDetails,
  goBack: navigateBack,
  goToProject,
  goToOrgRoute,
} as const;

export type ScyllaNavigate = typeof scyllaNavigate;
