import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { LoginForm } from './LoginForm';

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

describe('LoginForm', () => {
  it('the password field masks its input', () => {
    renderWithI18n(<LoginForm handleSubmit={vi.fn()} />);
    expect(screen.getByLabelText('Password')).toHaveAttribute('type', 'password');
  });

  it('Login stays disabled until both fields are filled', async () => {
    const user = userEvent.setup();
    renderWithI18n(<LoginForm handleSubmit={vi.fn()} />);
    const button = screen.getByRole('button', { name: 'Login' });
    expect(button).toBeDisabled();

    await user.type(screen.getByLabelText('Username'), 'ravenne');
    expect(button).toBeDisabled();

    await user.type(screen.getByLabelText('Password'), 'hunter2');
    expect(button).toBeEnabled();
  });

  it('submitting calls handleSubmit with the username and password', async () => {
    const handleSubmit = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<LoginForm handleSubmit={handleSubmit} />);

    await user.type(screen.getByLabelText('Username'), 'ravenne');
    await user.type(screen.getByLabelText('Password'), 'hunter2');
    await user.click(screen.getByRole('button', { name: 'Login' }));

    expect(handleSubmit).toHaveBeenCalledWith('ravenne', 'hunter2');
  });

  it('isPending disables the Login button even with valid input', () => {
    renderWithI18n(<LoginForm handleSubmit={vi.fn()} isPending />);
    expect(screen.getByRole('button', { name: 'Login' })).toBeDisabled();
  });
});
