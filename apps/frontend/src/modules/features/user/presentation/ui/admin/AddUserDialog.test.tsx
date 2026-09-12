import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { AddUserDialog } from './AddUserDialog';

const mutateMock = vi.fn();
vi.mock('@/modules/features/user/presentation/hooks/use-create-user.ts', () => ({
  useCreateUser: () => ({ mutate: mutateMock, isPending: false }),
}));

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

beforeEach(() => {
  mutateMock.mockReset();
});

describe('AddUserDialog', () => {
  it('Create stays disabled until both a username and a password are entered', async () => {
    const user = userEvent.setup();
    renderWithI18n(<AddUserDialog open setOpen={vi.fn()} />);
    const button = screen.getByRole('button', { name: 'Create User' });
    expect(button).toBeDisabled();

    await user.type(screen.getByLabelText('Username'), 'john.doe');
    expect(button).toBeDisabled();

    await user.type(screen.getByLabelText('Password'), 'hunter2');
    expect(button).toBeEnabled();
  });

  it('creates the user with the entered credentials and closes on success', async () => {
    mutateMock.mockImplementation((_vars, opts) => opts.onSuccess());
    const setOpen = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<AddUserDialog open setOpen={setOpen} />);

    await user.type(screen.getByLabelText('Username'), 'john.doe');
    await user.type(screen.getByLabelText('Password'), 'hunter2');
    await user.click(screen.getByRole('button', { name: 'Create User' }));

    expect(mutateMock).toHaveBeenCalledWith(
      { username: 'john.doe', password: 'hunter2' },
      expect.any(Object),
    );
    expect(setOpen).toHaveBeenCalledWith(false);
  });

  it('the password field masks its input', () => {
    renderWithI18n(<AddUserDialog open setOpen={vi.fn()} />);
    expect(screen.getByLabelText('Password')).toHaveAttribute('type', 'password');
  });
});
