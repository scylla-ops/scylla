import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import PermissionDenied from './PermissionDenied.svelte';

describe('PermissionDenied', () => {
  it('asks the user to contact an administrator by default', () => {
    render(PermissionDenied);

    expect(screen.getByText(/don't have the permission/i)).toBeInTheDocument();
    expect(screen.getByText(/ask an administrator/i)).toBeInTheDocument();
  });

  it('shows a custom message instead, when given one', () => {
    render(PermissionDenied, { message: 'You need the manage-roles permission.' });

    expect(screen.getByText('You need the manage-roles permission.')).toBeInTheDocument();
    expect(screen.queryByText(/ask an administrator/i)).not.toBeInTheDocument();
  });
});
