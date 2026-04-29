import { useLocation, useNavigate } from 'react-router-dom';
import { useContextStore } from '../stores/use-context.store.ts';
import type { Project } from '@/modules/features/project/domain/models/project.model.ts';

export const useScyllaNavigate = () => {
  const setProject = useContextStore(state => state.setProject);
  const setPipeline = useContextStore(state => state.setPipeline);
  const navigate = useNavigate();
  const { pathname } = useLocation();

  const goToSubRoute = (subPath: string, options = {}) => {
    const base = pathname.endsWith('/') ? pathname.slice(0, -1) : pathname;
    const cleanSubPath = subPath.startsWith('/') ? subPath.slice(1) : subPath;

    navigate(`${base}/${cleanSubPath}`, options);
  };

  const goToProject = (project: Project) => {
    navigate(`/projects/${project.id}`);
    setProject(project.id, project.name);
  };

  const goToCreatePipeline = () => {
    navigate(`/projects/${useContextStore.getState().project.id}/create`);
  };

  const goToEditPipeline = (id: string, name: string) => {
    navigate(`/projects/${useContextStore.getState().project.id}/edit/${id}`);
    setPipeline(id, name);
  };

  const goToJobs = (pipelineId: string) => {
    const projectId = useContextStore.getState().project.id;
    navigate(`/projects/${projectId}/pipelines/${pipelineId}/jobs`);
  };

  const goToUserSettings = (userId?: string) => {
    navigate('/users/' + (userId || 'me'));
  };

  return {
    navigate,
    goToEditPipeline,
    goToUserSettings,
    goToSubRoute,
    goToCreatePipeline,
    goToJobs,
    goBack: () => navigate(-1),
    goToProject,
  };
};
