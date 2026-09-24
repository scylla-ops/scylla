import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { installTestNavigator } from '@/test/navigator.ts';
import AuthGuardFixture from './AuthGuard.fixture.svelte';

let navigator: ReturnType<typeof installTestNavigator>;

beforeEach(() => {
  localStorage.clear();
  navigator = installTestNavigator();
});

afterEach(() => navigator.restore());

describe('AuthGuard', () => {
  it('redirects to /login when there is no token', () => {
    render(AuthGuardFixture);

    expect(navigator.navigate).toHaveBeenCalledWith('/login', { replace: true });
    expect(screen.queryByTestId('outlet')).not.toBeInTheDocument();
  });

  it('renders the routes when a token is present', () => {
    localStorage.setItem('token', 'a-real-token');
    render(AuthGuardFixture);

    expect(screen.getByTestId('outlet')).toBeInTheDocument();
    expect(navigator.navigate).not.toHaveBeenCalled();
  });

  it('still redirects for an empty-string token', () => {
    localStorage.setItem('token', '');
    render(AuthGuardFixture);

    expect(navigator.navigate).toHaveBeenCalledWith('/login', { replace: true });
  });
});
