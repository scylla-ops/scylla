import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { ConfirmOperationAlertDialog } from './ConfirmOperationAlertDialog';

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

describe('ConfirmOperationAlertDialog', () => {
  it('renders nothing when closed', () => {
    renderWithI18n(
      <ConfirmOperationAlertDialog open={false} onOpenChange={vi.fn()} onContinue={vi.fn()} />,
    );
    expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
  });

  it('shows the default title/description when open', () => {
    renderWithI18n(
      <ConfirmOperationAlertDialog open onOpenChange={vi.fn()} onContinue={vi.fn()} />,
    );
    expect(screen.getByText('Are you absolutely sure?')).toBeInTheDocument();
    expect(screen.getByText('This action cannot be undone.')).toBeInTheDocument();
  });

  it('shows a custom title/description instead', () => {
    renderWithI18n(
      <ConfirmOperationAlertDialog
        open
        onOpenChange={vi.fn()}
        onContinue={vi.fn()}
        title='Delete this secret?'
        description='It will stop working for every pipeline that uses it.'
      />,
    );
    expect(screen.getByText('Delete this secret?')).toBeInTheDocument();
    expect(screen.getByText('It will stop working for every pipeline that uses it.')).toBeInTheDocument();
  });

  it('calls onContinue when the Continue button is clicked', async () => {
    const onContinue = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(
      <ConfirmOperationAlertDialog open onOpenChange={vi.fn()} onContinue={onContinue} />,
    );
    await user.click(screen.getByRole('button', { name: 'Continue' }));
    expect(onContinue).toHaveBeenCalled();
  });

  it('calls onOpenChange(false) when Cancel is clicked', async () => {
    const onOpenChange = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(
      <ConfirmOperationAlertDialog open onOpenChange={onOpenChange} onContinue={vi.fn()} />,
    );
    await user.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it('disables both buttons while isLoading', () => {
    renderWithI18n(
      <ConfirmOperationAlertDialog open onOpenChange={vi.fn()} onContinue={vi.fn()} isLoading />,
    );
    expect(screen.getByRole('button', { name: 'Cancel' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Continue' })).toBeDisabled();
  });
});
