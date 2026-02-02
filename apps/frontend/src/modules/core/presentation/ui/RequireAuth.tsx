import { Navigate, Outlet } from 'react-router-dom';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';

export const RequireAuth = () => {
  const deps = useDependencies();
  let token = deps.core.getTokenUseCase.execute();

  if (!token && import.meta.env.DEV) {
    token = 'dev-mock-token';
  }

  if (!token) {
    console.log('notoken');
    return <Navigate to='/login' replace />;
  }
  console.log(token);

  return <Outlet />;
};
