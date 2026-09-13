import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithI18n } from '@/test/render.tsx';
import userEvent from '@testing-library/user-event';
import { ConfirmOperationAlertDialog } from './ConfirmOperationAlertDialog';

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
