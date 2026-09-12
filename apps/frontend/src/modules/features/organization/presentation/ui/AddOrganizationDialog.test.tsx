import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { useContextStore } from '@platform/context';
import { AddOrganizationDialog } from './AddOrganizationDialog';

const navigateMock = vi.fn();
vi.mock('react-router-dom', () => ({
  useNavigate: () => navigateMock,
}));

const mutateMock = vi.fn();
const pendingState = { isPending: false };
vi.mock(
  '@/modules/features/organization/presentation/hooks/useCreateOrganization.ts',
  () => ({
    useCreateOrganization: () => ({ mutate: mutateMock, isPending: pendingState.isPending }),
  }),
);

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

beforeEach(() => {
  navigateMock.mockClear();
  mutateMock.mockReset();
  pendingState.isPending = false;
  useContextStore.setState({ organization: { id: null, name: null } });
});

describe('AddOrganizationDialog', () => {
  it('the Create button stays disabled until both name and description are entered', async () => {
    const user = userEvent.setup();
    renderWithI18n(<AddOrganizationDialog open setOpen={vi.fn()} />);
    const button = screen.getByRole('button', { name: 'Create Organization' });
    expect(button).toBeDisabled();

    // Description isn't marked optional on this form either, even though the
    // dialog's own submit guard only actually cares about the name.
    await user.type(screen.getByLabelText('Organization name'), 'Acme Corp');
    expect(button).toBeDisabled();

    await user.type(screen.getByLabelText('Description'), 'a real one');
    expect(button).toBeEnabled();
  });

  it('submits the raw (untrimmed) name and reacts to success', async () => {
    mutateMock.mockImplementation((_vars, opts) => opts.onSuccess({ id: 'org-1' }));
    const setOpen = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<AddOrganizationDialog open setOpen={setOpen} />);

    await user.type(screen.getByLabelText('Organization name'), '  Acme Corp  ');
    await user.type(screen.getByLabelText('Description'), 'a real one');
    await user.click(screen.getByRole('button', { name: 'Create Organization' }));

    // Unlike the description, the name reaches the mutation exactly as typed -
    // only the submit guard (name.trim()) and the checklist's own validity
    // check look at a trimmed copy.
    expect(mutateMock).toHaveBeenCalledWith(
      { name: '  Acme Corp  ', description: 'a real one' },
      expect.any(Object),
    );
    expect(setOpen).toHaveBeenCalledWith(false);
    expect(useContextStore.getState().organization).toEqual({ id: 'org-1', name: '  Acme Corp  ' });
    expect(navigateMock).toHaveBeenCalledWith('/acme-corp/projects');
  });

  it('sends a trimmed, non-empty description', async () => {
    mutateMock.mockImplementation((_vars, opts) => opts.onSuccess({ id: 'org-2' }));
    const user = userEvent.setup();
    renderWithI18n(<AddOrganizationDialog open setOpen={vi.fn()} />);

    await user.type(screen.getByLabelText('Organization name'), 'Acme');
    await user.type(screen.getByLabelText('Description'), '  a real one  ');
    await user.click(screen.getByRole('button', { name: 'Create Organization' }));

    expect(mutateMock).toHaveBeenCalledWith(
      { name: 'Acme', description: 'a real one' },
      expect.any(Object),
    );
  });

  it('hides the Cancel button when hideCancel is set', () => {
    renderWithI18n(<AddOrganizationDialog open setOpen={vi.fn()} hideCancel />);
    expect(screen.queryByRole('button', { name: 'Cancel' })).not.toBeInTheDocument();
  });
});
