import { useContextStore } from '@platform/context';

/** Ends the session and reloads the app on the login page. */
export const signOut = (): void => {
  localStorage.removeItem('token');
  useContextStore.getState().reset();
  window.location.href = '/login';
};
