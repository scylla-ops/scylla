import { describe, it, expect, vi } from 'vitest';
import { fireEvent, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { render } from '@/test/render.svelte.ts';
import AlertDialogFixture from './alert-dialog.fixture.svelte';

describe('AlertDialog', () => {
  it('renders nothing when closed', () => {
    render(AlertDialogFixture, { open: false });

    expect(screen.queryByRole('alertdialog')).toBeNull();
  });

  it('is an alertdialog, not a dialog — which the React original never was', async () => {
    render(AlertDialogFixture, { open: true });

    const dialog = await screen.findByRole('alertdialog');
    expect(dialog).toHaveAccessibleName('Are you absolutely sure?');
    expect(dialog).toHaveAccessibleDescription('This action cannot be undone.');
  });

  it('calls onContinue from the action button', async () => {
    const onContinue = vi.fn();
    render(AlertDialogFixture, { open: true, onContinue });

    await userEvent.click(await screen.findByRole('button', { name: 'Continue' }));

    expect(onContinue).toHaveBeenCalledOnce();
  });

  it('stays open after Continue, so a pending mutation is not dismissed', async () => {
    render(AlertDialogFixture, { open: true, onContinue: vi.fn() });

    await userEvent.click(await screen.findByRole('button', { name: 'Continue' }));

    // The parent owns `open`. bits-ui's own Action would have closed it here,
    // which is why the port uses plain buttons.
    expect(screen.getByRole('alertdialog')).toBeInTheDocument();
  });

  it('calls onCancel from the cancel button', async () => {
    const onCancel = vi.fn();
    render(AlertDialogFixture, { open: true, onCancel });

    await userEvent.click(await screen.findByRole('button', { name: 'Cancel' }));

    expect(onCancel).toHaveBeenCalledOnce();
  });

  it('disables both buttons while the operation runs', async () => {
    render(AlertDialogFixture, { open: true, isLoading: true });

    expect(await screen.findByRole('button', { name: 'Cancel' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Continue' })).toBeDisabled();
  });

  it('does not dismiss on a click outside, unlike a plain dialog', async () => {
    render(AlertDialogFixture, { open: true });
    await screen.findByRole('alertdialog');

    // `fireEvent`, not `userEvent`: the open dialog puts `pointer-events: none`
    // on `<body>`, which userEvent refuses to click through. The dismiss layer
    // listens for a document-level `pointerdown`, so that is what an outside
    // click is from its point of view.
    await fireEvent.pointerDown(document.body);

    expect(screen.getByRole('alertdialog')).toBeInTheDocument();
  });
});
