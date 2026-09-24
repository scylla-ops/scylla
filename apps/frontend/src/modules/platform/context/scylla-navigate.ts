import { currentPathname, navigateBack, navigateTo } from './navigator.ts';
import { contextStore } from './context.store.ts';
import { slugifyOrgName } from '@shared/utils/slug.ts';

/** Builds the URL from the current organization and project, read at call time. */

const getOrgPrefix = () => {
  const orgName = contextStore.getState().organization.name;
  return orgName ? `/${slugifyOrgName(orgName)}` : '';
};

const goToSubRoute = (subPath: string, options = {}) => {
  const pathname = currentPathname();
  const base = pathname.endsWith('/') ? pathname.slice(0, -1) : pathname;
  const cleanSubPath = subPath.startsWith('/') ? subPath.slice(1) : subPath;
  navigateTo(`${base}/${cleanSubPath}`, options);
};

const goToProject = (id: string, name: string) => {
  navigateTo(`${getOrgPrefix()}/projects/${id}`);
  contextStore.getState().setProject(id, name);
};

const goToCreatePipeline = () => {
  navigateTo(`${getOrgPrefix()}/projects/${contextStore.getState().project.id}/create`);
};

const goToEditPipeline = (id: string, name: string) => {
  navigateTo(`${getOrgPrefix()}/projects/${contextStore.getState().project.id}/edit/${id}`);
  contextStore.getState().setPipeline(id, name);
};

// No `name`: the caller is already inside this pipeline.
const goToJobs = (id: string, name?: string) => {
  const projectId = contextStore.getState().project.id;
  navigateTo(`${getOrgPrefix()}/projects/${projectId}/pipelines/${id}/jobs`);
  if (name) contextStore.getState().setPipeline(id, name);
};

const goToJobDetails = (
  pipelineId: string,
  jobId: string,
  options: { nodeId?: string; pipelineName?: string } = {},
) => {
  const projectId = contextStore.getState().project.id;
  const query = options.nodeId ? `?nodes=${encodeURIComponent(options.nodeId)}` : '';
  navigateTo(
    `${getOrgPrefix()}/projects/${projectId}/pipelines/${pipelineId}/jobs/${jobId}${query}`,
  );
  if (options.pipelineName)
    contextStore.getState().setPipeline(pipelineId, options.pipelineName);
};

const goToTriggers = (id: string, name: string) => {
  const projectId = contextStore.getState().project.id;
  navigateTo(`${getOrgPrefix()}/projects/${projectId}/pipelines/${id}/triggers`);
  contextStore.getState().setPipeline(id, name);
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
