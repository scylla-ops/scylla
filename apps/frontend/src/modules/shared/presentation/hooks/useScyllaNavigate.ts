import { useLocation, useNavigate } from 'react-router-dom';

export const useScyllaNavigate = () => {
  const navigate = useNavigate();
  const { pathname } = useLocation();

  const goToSubRoute = (subPath: string, options = {}) => {
    // On nettoie pour éviter le double //
    const base = pathname.endsWith('/') ? pathname.slice(0, -1) : pathname;
    const cleanSubPath = subPath.startsWith('/') ? subPath.slice(1) : subPath;

    navigate(`${base}/${cleanSubPath}`, options);
  };

  const goToProjectCreate = (projectId: string) => {
    navigate(`/projects/${projectId}/create`);
  };

  return {
    navigate,
    goToSubRoute,
    goToProjectCreate,
    goBack: () => navigate(-1),
  };
};
