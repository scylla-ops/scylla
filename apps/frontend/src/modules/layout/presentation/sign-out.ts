import { contextStore } from '@platform/context';

export const signOut = (): void => {
  localStorage.removeItem('token');
  contextStore.getState().reset();
  window.location.href = '/login';
};
