import { useContextStore } from '@/modules/shared/presentation/stores/useContext';
import { useEffect } from 'react';
import { Outlet, useLocation } from 'react-router-dom';

/**Middleware sed to clean the context store depending on the actual route */
export const ContextCleaner = () => {
  const location = useLocation();

  const setProject = useContextStore(state => state.setProject);
  const setPipeline = useContextStore(state => state.setPipeline);

  useEffect(() => {
    const { project, pipeline } = useContextStore.getState();

    if (project?.id && !location.pathname.includes(project.id)) {
      setProject(null, null);
    }

    if (pipeline?.id && !location.pathname.includes('/edit/')) {
      console.log('Suppression du pipeline car on a quitté la page édit');
      setPipeline(null, null);
    }
  }, [location.pathname, setPipeline, setProject]);

  return <Outlet />;
};
