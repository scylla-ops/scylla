import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { Dialog } from '@shadcn';
import { RoleDialogHeader } from './RoleDialogHeader';

// DialogTitle/DialogDescription read their context from an ancestor Dialog root.
const renderWithI18n = (ui: React.ReactElement) =>
  render(
    <I18nProvider i18n={i18n}>
      <Dialog open>{ui}</Dialog>
    </I18nProvider>,
  );

describe('RoleDialogHeader', () => {
  it('titles itself "Create role" when not editing', () => {
    renderWithI18n(<RoleDialogHeader isEdit={false} />);
    expect(screen.getByText('Create role')).toBeInTheDocument();
  });

  it('titles itself "Edit role" when editing', () => {
    renderWithI18n(<RoleDialogHeader isEdit />);
    expect(screen.getByText('Edit role')).toBeInTheDocument();
  });
});
