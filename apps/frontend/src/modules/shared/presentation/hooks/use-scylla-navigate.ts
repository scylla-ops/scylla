import { useLocation, useNavigate } from 'react-router-dom';
import { useContextStore } from '../stores/use-context.store.ts';
import type { Project } from '@/modules/features/project/domain/models/project.model.ts';
import type { PipelineIdentity } from '@/modules/features/pipeline/domain/models/pipeline.model.ts';
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

    navigate(`${base}/${cleanSubPath}`, options);
  };

  const goToProject = (project: Project) => {
    navigate(`${getOrgPrefix()}/projects/${project.id}`);
    setProject(project.id, project.name);
  };

  const goToCreatePipeline = () => {
    navigate(`${getOrgPrefix()}/projects/${useContextStore.getState().project.id}/create`);
  };

  const goToEditPipeline = (id: string, name: string) => {
    navigate(`${getOrgPrefix()}/projects/${useContextStore.getState().project.id}/edit/${id}`);
    setPipeline(id, name);
  };

  const goToJobs = (pipeline: PipelineIdentity) => {
    const projectId = useContextStore.getState().project.id;
    navigate(`${getOrgPrefix()}/projects/${projectId}/pipelines/${pipeline.id}/jobs`);
    setPipeline(pipeline.id, pipeline.name);
  };

  const goToUserSettings = (userId?: string) => {
    navigate(`${getOrgPrefix()}/users/${userId || 'me'}`, { replace: true });
  };

  const goToAgentDetails = (agentId: string) => {
    navigate(`${getOrgPrefix()}/agents/${agentId}`);
  };

  const goToOrgRoute = (path: string) => {
    navigate(`${getOrgPrefix()}${path.startsWith('/') ? path : `/${path}`}`);
  };

  const goBack = () => navigate(-1);

  return {
    navigate,
    goToEditPipeline,
    goToUserSettings,
    goToSubRoute,
    goToCreatePipeline,
    goToJobs,
    goToAgentDetails,
    goBack: () => navigate(-1),
    goToProject,
    goToOrgRoute,
  };
};
