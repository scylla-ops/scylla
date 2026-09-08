import { useLocation, useNavigate } from 'react-router-dom';
import { useContextStore } from './use-context.store.ts';
import { slugifyOrgName } from '@shared/utils/slug.ts';

export const useScyllaNavigate = () => {
  const setProject = useContextStore(state => state.setProject);
  const setPipeline = useContextStore(state => state.setPipeline);
  const navigate = useNavigate();
  const { pathname } = useLocation();

  const getOrgPrefix = () => {
    const orgName = useContextStore.getState().organization.name;
    return orgName ? `/${slugifyOrgName(orgName)}` : '';
  };

  const goToSubRoute = (subPath: string, options = {}) => {
    const base = pathname.endsWith('/') ? pathname.slice(0, -1) : pathname;
    const cleanSubPath = subPath.startsWith('/') ? subPath.slice(1) : subPath;
    void navigate(`${base}/${cleanSubPath}`, options);
  };

  // Navigation takes ids and names, never feature entities: this hook is shared
  // infrastructure and must stay ignorant of what a project or a pipeline is.
  const goToProject = (id: string, name: string) => {
    void navigate(`${getOrgPrefix()}/projects/${id}`);
    setProject(id, name);
  };

  const goToCreatePipeline = () => {
    void navigate(`${getOrgPrefix()}/projects/${useContextStore.getState().project.id}/create`);
  };

  const goToEditPipeline = (id: string, name: string) => {
    void navigate(`${getOrgPrefix()}/projects/${useContextStore.getState().project.id}/edit/${id}`);
    setPipeline(id, name);
  };

  const goToJobs = (id: string, name: string) => {
    const projectId = useContextStore.getState().project.id;
    void navigate(`${getOrgPrefix()}/projects/${projectId}/pipelines/${id}/jobs`);
    setPipeline(id, name);
  };

  const goToTriggers = (id: string, name: string) => {
    const projectId = useContextStore.getState().project.id;
    void navigate(`${getOrgPrefix()}/projects/${projectId}/pipelines/${id}/triggers`);
    setPipeline(id, name);
  };

  const goToUserSettings = (userId: string) => {
    void navigate(`${getOrgPrefix()}/users/${userId}`, { replace: true });
  };

  const goToAgentDetails = (agentId: string) => {
    void navigate(`${getOrgPrefix()}/agents/${agentId}`);
  };

  const goToOrgRoute = (path: string) => {
    void navigate(`${getOrgPrefix()}${path.startsWith('/') ? path : `/${path}`}`);
  };

  return {
    navigate,
    goToEditPipeline,
    goToUserSettings,
    goToSubRoute,
    goToCreatePipeline,
    goToJobs,
    goToTriggers,
    goToAgentDetails,
    goBack: () => void navigate(-1),
    goToProject,
    goToOrgRoute,
  };
};
