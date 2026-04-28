import type { ProjectResponse } from '@/generated/project';
import { useLocation, useNavigate } from 'react-router-dom';
import { useContextStore } from '../stores/use-context.store.ts';
import type { PipelineSummary } from '@/generated/pipeline';

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

  const goToProject = (project: ProjectResponse) => {
    navigate(`/projects/${project.projectId}`);
    setProject(project.projectId, project.name);
  };

  const goToCreatePipeline = () => {
    navigate(`/projects/${useContextStore.getState().project.id}/create`);
  };

  const goToEditPipeline = (pipeline: PipelineSummary) => {
    navigate(`/projects/${useContextStore.getState().project.id}/edit/${pipeline.pipelineId}`);
    setPipeline(pipeline.pipelineId, pipeline.name);
  };

  const goToJobs = (pipeline: PipelineSummary) => {
    const projectId = useContextStore.getState().project.id;
    navigate(`/projects/${projectId}/pipelines/${pipeline.pipelineId}/jobs`);
    setPipeline(pipeline.pipelineId, pipeline.name);
  };

  const goToUserSettings = (userId?: string) => {
    navigate('/users/' + (userId || 'me'));
  };

  const goToWorkerDetails = (workerId: string) => {
    navigate(`/workers/${workerId}`);
  };

  return {
    navigate,
    goToEditPipeline,
    goToUserSettings,
    goToSubRoute,
    goToCreatePipeline,
    goToJobs,
    goToWorkerDetails,
    goBack: () => navigate(-1),
    goToProject,
  };
};
