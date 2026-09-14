import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithI18n } from '@/test/render.tsx';
import { PermissionDenied } from './PermissionDenied';

describe('PermissionDenied', () => {
  it('shows the generic "ask an administrator" message by default', () => {
    renderWithI18n(<PermissionDenied />);
    expect(screen.getByText(/don't have the permission/i)).toBeInTheDocument();
    expect(screen.getByText(/ask an administrator/i)).toBeInTheDocument();
  });

  it('shows a custom message instead, when given', () => {
    renderWithI18n(<PermissionDenied message='You need the manage-roles permission.' />);
    expect(screen.getByText('You need the manage-roles permission.')).toBeInTheDocument();
    expect(screen.queryByText(/ask an administrator/i)).not.toBeInTheDocument();
  });
});
