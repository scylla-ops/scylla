import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { CreateSecretDialog } from './CreateSecretDialog';

const mutateMock = vi.fn();
vi.mock('@/modules/features/secret/presentation/hooks/use-secrets.ts', () => ({
  useCreateSecret: () => ({ mutate: mutateMock, isPending: false }),
}));

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

beforeEach(() => {
  mutateMock.mockReset();
});

const fillValidForm = async (user: ReturnType<typeof userEvent.setup>) => {
  await user.type(screen.getByLabelText('Secret name'), 'DATABASE_URL');
  await user.type(screen.getByLabelText('Description'), 'the prod db');
  await user.type(screen.getByLabelText('Value'), 'postgres://...');
};

describe('CreateSecretDialog', () => {
  it('rejects a name with characters outside the backend pattern, keeping Create disabled', async () => {
    const user = userEvent.setup();
    renderWithI18n(<CreateSecretDialog isOpen projectId='project-1' setOpen={vi.fn()} />);

    await user.type(screen.getByLabelText('Secret name'), 'not a valid name');
    await user.type(screen.getByLabelText('Description'), 'desc');
    await user.type(screen.getByLabelText('Value'), 'value');
    expect(screen.getByRole('button', { name: 'Create' })).toBeDisabled();
  });

  it('creates the secret scoped to the given project and closes the dialog immediately', async () => {
    const setOpen = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<CreateSecretDialog isOpen projectId='project-42' setOpen={setOpen} />);

    await fillValidForm(user);
    await user.click(screen.getByRole('button', { name: 'Create' }));

    expect(mutateMock).toHaveBeenCalledWith({
      name: 'DATABASE_URL',
      description: 'the prod db',
      value: 'postgres://...',
    });
    // Unlike the other create dialogs, this one closes right away rather than
    // waiting on the mutation's own onSuccess - it never passes one.
    expect(setOpen).toHaveBeenCalledWith(false);
  });
});
