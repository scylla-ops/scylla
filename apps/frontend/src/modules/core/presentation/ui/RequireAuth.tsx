import { Navigate, Outlet } from 'react-router-dom';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';

export const RequireAuth = () => {
  const deps = useDependencies();
  const token = deps.core.getTokenUseCase.execute();

  if (!token) {
    return <Navigate to='/login' replace />;
  }

  return <Outlet />;
};
