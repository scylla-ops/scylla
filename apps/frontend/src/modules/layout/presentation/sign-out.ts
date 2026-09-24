import { contextStore } from '@platform/context';

/** Ends the session and reloads the app on the login page. */
export const signOut = (): void => {
  localStorage.removeItem('token');
  contextStore.getState().reset();
  window.location.href = '/login';
};
