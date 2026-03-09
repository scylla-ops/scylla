import { Navigate, Outlet } from 'react-router-dom';

export const RequireAuth = () => {
  const token = localStorage.getItem('token');

  if (!token) {
    console.log('notoken');
    return <Navigate to='/login' replace />;
  }
  console.log(token);

  return <Outlet />;
};
