import { Navigate, Outlet } from 'react-router-dom';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';

export const RequireAuth = () => {
  const deps = useDependencies();
  const token = deps.core.getTokenUseCase.execute();

  if (!token) {
    console.log('notoken');
    return <Navigate to='/login' replace />;
  }
  console.log(token);

  return <Outlet />;
};
