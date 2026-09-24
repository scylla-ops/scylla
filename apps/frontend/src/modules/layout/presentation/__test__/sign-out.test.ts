import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { contextStore } from '@platform/context';
import { signOut } from '../sign-out.ts';

const location = { href: '/acme/dashboard' };

beforeEach(() => {
  vi.stubGlobal('location', location);
  localStorage.setItem('token', 'a-token');
  contextStore.setState({ organization: { id: 'org-1', name: 'Acme' } });
});

afterEach(() => vi.unstubAllGlobals());

describe('signOut', () => {
  it('removes the token, clears the context and loads the login page', () => {
    signOut();

    expect(localStorage.getItem('token')).toBeNull();
    expect(contextStore.getState().organization).toEqual({ id: null, name: null });
    expect(location.href).toBe('/login');
  });
});
