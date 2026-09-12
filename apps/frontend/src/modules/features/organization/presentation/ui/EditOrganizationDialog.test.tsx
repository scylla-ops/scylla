import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { EditOrganizationDialog } from './EditOrganizationDialog';

const mutateMock = vi.fn();
vi.mock('@/modules/features/organization/presentation/hooks/use-update-organization.ts', () => ({
  useUpdateOrganization: () => ({ mutate: mutateMock, isPending: false }),
}));

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

beforeEach(() => {
  mutateMock.mockReset();
});

describe('EditOrganizationDialog', () => {
  it('starts with blank fields (the current name/description are only shown as placeholders) so Save starts disabled', () => {
    renderWithI18n(
      <EditOrganizationDialog
        open
        setOpen={vi.fn()}
        organization={{ id: 'org-1', name: 'Acme Corp', description: 'a real one' }}
      />,
    );
    expect(screen.getByLabelText('Organization name')).toHaveValue('');
    expect(screen.getByLabelText('Organization name')).toHaveAttribute('placeholder', 'Acme Corp');
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
  });

  it('falls back to a generic placeholder when there is no existing description', () => {
    renderWithI18n(
      <EditOrganizationDialog open setOpen={vi.fn()} organization={{ id: 'org-1', name: 'Acme' }} />,
    );
    expect(screen.getByLabelText('Description')).toHaveAttribute(
      'placeholder',
      'Add a description...',
    );
  });

  it('submits the trimmed new name/description for this organization id and closes on success', async () => {
    mutateMock.mockImplementation((_vars, opts) => opts.onSuccess());
    const setOpen = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(
      <EditOrganizationDialog
        open
        setOpen={setOpen}
        organization={{ id: 'org-9', name: 'Acme Corp' }}
      />,
    );

    await user.type(screen.getByLabelText('Organization name'), '  New Name  ');
    await user.type(screen.getByLabelText('Description'), '  new desc  ');
    await user.click(screen.getByRole('button', { name: 'Save' }));

    expect(mutateMock).toHaveBeenCalledWith(
      { organizationId: 'org-9', name: 'New Name', description: 'new desc' },
      expect.any(Object),
    );
    expect(setOpen).toHaveBeenCalledWith(false);
  });
});
