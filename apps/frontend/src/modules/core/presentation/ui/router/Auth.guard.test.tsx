import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { AuthGuard } from './Auth.guard';

vi.mock('react-router-dom', () => ({
  Navigate: ({ to }: { to: string }) => <div data-testid='navigate' data-to={to} />,
  Outlet: () => <div data-testid='outlet' />,
}));

beforeEach(() => {
  localStorage.clear();
});

describe('AuthGuard', () => {
  it('redirects to /login when there is no token', () => {
    render(<AuthGuard />);
    const navigate = screen.getByTestId('navigate');
    expect(navigate).toHaveAttribute('data-to', '/login');
    expect(screen.queryByTestId('outlet')).not.toBeInTheDocument();
  });

  it('renders the outlet when a token is present', () => {
    localStorage.setItem('token', 'a-real-token');
    render(<AuthGuard />);
    expect(screen.getByTestId('outlet')).toBeInTheDocument();
    expect(screen.queryByTestId('navigate')).not.toBeInTheDocument();
  });

  it('still redirects for an empty-string token', () => {
    localStorage.setItem('token', '');
    render(<AuthGuard />);
    expect(screen.getByTestId('navigate')).toHaveAttribute('data-to', '/login');
  });
});
