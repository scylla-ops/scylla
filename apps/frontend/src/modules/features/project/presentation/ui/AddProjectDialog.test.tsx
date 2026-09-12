import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { useContextStore } from '@platform/context';
import { AddProjectDialog } from './AddProjectDialog';

const mutateMock = vi.fn();
vi.mock('@/modules/features/project/presentation/hooks/useCreateProject.ts', () => ({
  useCreateProject: () => ({ mutate: mutateMock, isPending: false }),
}));

const toastError = vi.fn();
vi.mock('@shared/presentation/utils/toast.ts', () => ({
  toast: { error: (...args: unknown[]) => toastError(...args) },
}));

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

beforeEach(() => {
  mutateMock.mockReset();
  toastError.mockClear();
  useContextStore.setState({ organization: { id: 'org-1', name: 'Acme' } });
});

describe('AddProjectDialog', () => {
  it('toasts an error and never calls the mutation when no organization is selected', async () => {
    useContextStore.setState({ organization: { id: null, name: null } });
    const user = userEvent.setup();
    renderWithI18n(<AddProjectDialog open setOpen={vi.fn()} />);

    // Description isn't marked optional either, so both fields are needed just
    // to get the Create button enabled - the organization check only kicks in
    // once the form itself considers the input valid.
    await user.type(screen.getByLabelText('Project name'), 'web');
    await user.type(screen.getByLabelText('Description'), 'the web app');
    await user.click(screen.getByRole('button', { name: 'Create Project' }));

    expect(toastError).toHaveBeenCalledWith(
      'Project name is required and you must select an organization.',
    );
    expect(mutateMock).not.toHaveBeenCalled();
  });

  it('creates the project under the current organization and closes on success', async () => {
    mutateMock.mockImplementation((_vars, opts) => opts.onSuccess());
    const setOpen = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<AddProjectDialog open setOpen={setOpen} />);

    await user.type(screen.getByLabelText('Project name'), 'web');
    await user.type(screen.getByLabelText('Description'), '  the web app  ');
    await user.click(screen.getByRole('button', { name: 'Create Project' }));

    expect(mutateMock).toHaveBeenCalledWith(
      { name: 'web', organizationId: 'org-1', description: 'the web app' },
      expect.any(Object),
    );
    expect(setOpen).toHaveBeenCalledWith(false);
  });

  it('a description of only whitespace is trimmed down to undefined - but also fails the form\'s own validity check, so Create never enables', async () => {
    const user = userEvent.setup();
    renderWithI18n(<AddProjectDialog open setOpen={vi.fn()} />);

    await user.type(screen.getByLabelText('Project name'), 'web');
    await user.type(screen.getByLabelText('Description'), '   ');
    expect(screen.getByRole('button', { name: 'Create Project' })).toBeDisabled();
  });
});
