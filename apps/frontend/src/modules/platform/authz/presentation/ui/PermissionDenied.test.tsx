import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { PermissionDenied } from './PermissionDenied';

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

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
