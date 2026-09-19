import { useContextStore } from '@platform/context';
import { useEffect } from 'react';
import { Outlet, useLocation, useNavigate, useParams } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { projectQueries } from '@/modules/features/project';
import { slugifyOrgName } from '@shared/utils/slug.ts';

/**Middleware used to clean the context store depending on the actual route */
export const ContextCleanerWrapper = () => {
  const location = useLocation();
  const { projectId } = useParams<{ projectId: string }>();
  const navigate = useNavigate();

  const setProject = useContextStore(state => state.setProject);
  const setPipeline = useContextStore(state => state.setPipeline);
  const organizationId = useContextStore(state => state.organization.id);
  const organizationName = useContextStore(state => state.organization.name);

  const { data, isLoading } = useQuery(projectQueries.byOrganization(organizationId));
  const projects = data?.projects;

  useEffect(() => {
    if (isLoading || !projects || !projectId) return;

    const projectExists = projects.some(p => p.id === projectId);

    if (!projectExists) {
      setProject(null, null);
      setPipeline(null, null);
      const slug = organizationName ? slugifyOrgName(organizationName) : '';
      void navigate(slug ? `/${slug}/projects` : '/', { replace: true });
      return;
    }

    const { pipeline } = useContextStore.getState();
    if (
      pipeline?.id &&
      !location.pathname.includes('/edit/') &&
      !location.pathname.includes('/pipelines/')
    ) {
      setPipeline(null, null);
    }
  }, [
    projectId,
    projects,
    isLoading,
    navigate,
    setProject,
    setPipeline,
    organizationName,
    location.pathname,
  ]);

  return <Outlet />;
};
