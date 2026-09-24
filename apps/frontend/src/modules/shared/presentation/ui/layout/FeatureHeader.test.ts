import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { findTooltip, render, textSnippet } from '@/test/render.svelte.ts';
import FeatureHeader from './FeatureHeader.svelte';

const toastSuccess = vi.fn();
vi.mock('svelte-sonner', () => ({
  toast: { success: (...args: unknown[]) => toastSuccess(...args) },
}));

beforeEach(() => toastSuccess.mockClear());

/**
 * Unlike the React original, the delete button carries an `sr-only` label, so it
 * is reachable by role and name like everything else — no `querySelector` on a
 * `data-variant`.
 */
const deleteButton = () => screen.getByRole('button', { name: 'Delete' });

describe('FeatureHeader', () => {
  it('shows the count and the singular label', () => {
    render(FeatureHeader, { count: 1, label: 'Secret' });

    expect(screen.getByRole('heading')).toHaveTextContent('1 Secret');
    expect(screen.getByText('in total')).toBeInTheDocument();
  });

  it('switches to the plural label once count > 1', () => {
    render(FeatureHeader, { count: 4, label: 'Secret', pluralLabel: 'Secrets' });

    expect(screen.getByRole('heading')).toHaveTextContent('4 Secrets');
  });

  it('falls back to the singular label when no plural was given, even with count > 1', () => {
    render(FeatureHeader, { count: 4, label: 'Secret' });

    expect(screen.getByRole('heading')).toHaveTextContent('4 Secret');
  });

  it('omits the count block entirely when no count was given', () => {
    render(FeatureHeader, { label: 'Secret' });

    expect(screen.queryByText('in total')).toBeNull();
  });

  it('omits "Select all" once everything is already selected', () => {
    render(FeatureHeader, {
      count: 3,
      label: 'Secret',
      onSelectAll: vi.fn(),
      allSelected: true,
    });

    expect(screen.queryByRole('button', { name: 'Select all' })).toBeNull();
  });

  it('shows "Clear" only once something is selected', () => {
    const { rerender } = render(FeatureHeader, {
      label: 'Secret',
      selectedCount: 0,
      onClearSelection: vi.fn(),
    });
    expect(screen.queryByRole('button', { name: 'Clear' })).toBeNull();

    void rerender({ label: 'Secret', selectedCount: 2, onClearSelection: vi.fn() });

    expect(screen.getByRole('button', { name: 'Clear' })).toBeInTheDocument();
  });

  it('calls onNew when New is clicked', async () => {
    const onNew = vi.fn();
    render(FeatureHeader, { label: 'Secret', onNew });

    await userEvent.click(screen.getByRole('button', { name: 'New Secret' }));

    expect(onNew).toHaveBeenCalledOnce();
  });

  it('disables New with an explaining tooltip when canNew is false', async () => {
    render(FeatureHeader, { label: 'Secret', onNew: vi.fn(), canNew: false });

    const button = screen.getByRole('button', { name: 'New Secret' });
    expect(button).toBeDisabled();

    await userEvent.hover(button.parentElement as HTMLElement);

    expect(await findTooltip()).toHaveTextContent("You don't have permission to do this.");
  });

  it('shows a caller-supplied reason instead of the generic one', async () => {
    render(FeatureHeader, {
      label: 'Secret',
      onNew: vi.fn(),
      canNew: false,
      newDeniedReason: 'Ask an org admin for CREATE_SECRETS.',
    });

    await userEvent.hover(
      screen.getByRole('button', { name: 'New Secret' }).parentElement as HTMLElement,
    );

    expect(await findTooltip()).toHaveTextContent('Ask an org admin for CREATE_SECRETS.');
  });

  it('opens a confirmation dialog before deleting, then toasts on success', async () => {
    const onDeleteSelection = vi.fn().mockResolvedValue(undefined);
    render(FeatureHeader, { label: 'Secret', selectedCount: 3, onDeleteSelection });

    await userEvent.click(deleteButton());
    await screen.findByRole('alertdialog');
    expect(onDeleteSelection).not.toHaveBeenCalled();

    await userEvent.click(screen.getByRole('button', { name: 'Continue' }));

    await waitFor(() => expect(onDeleteSelection).toHaveBeenCalledOnce());
    expect(toastSuccess).toHaveBeenCalledWith('3 items deleted');
  });

  it('singularizes the delete-success toast for exactly one item', async () => {
    render(FeatureHeader, {
      label: 'Secret',
      selectedCount: 1,
      onDeleteSelection: vi.fn().mockResolvedValue(undefined),
    });

    await userEvent.click(deleteButton());
    await userEvent.click(await screen.findByRole('button', { name: 'Continue' }));

    await waitFor(() => expect(toastSuccess).toHaveBeenCalledWith('1 item deleted'));
  });

  it('does not toast when the delete call fails, and closes the dialog anyway', async () => {
    render(FeatureHeader, {
      label: 'Secret',
      selectedCount: 2,
      onDeleteSelection: vi.fn().mockRejectedValue(new Error('denied')),
    });

    await userEvent.click(deleteButton());
    await userEvent.click(await screen.findByRole('button', { name: 'Continue' }));

    // The error toast is the global MutationCache handler's job, not this one's.
    await waitFor(() => expect(screen.queryByRole('alertdialog')).toBeNull());
    expect(toastSuccess).not.toHaveBeenCalled();
  });

  it('disables the delete action with a tooltip when canDelete is false', async () => {
    render(FeatureHeader, {
      label: 'Secret',
      selectedCount: 2,
      onDeleteSelection: vi.fn(),
      canDelete: false,
      deleteDeniedReason: 'Deleting secrets needs DELETE_SECRETS.',
    });

    expect(deleteButton()).toBeDisabled();

    await userEvent.hover(deleteButton().parentElement as HTMLElement);

    expect(await findTooltip()).toHaveTextContent('Deleting secrets needs DELETE_SECRETS.');
  });

  it('renders extraActions alongside the built-in ones', () => {
    render(FeatureHeader, {
      label: 'Secret',
      onNew: vi.fn(),
      extraActions: textSnippet('Import'),
    });

    expect(screen.getByText('Import')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'New Secret' })).toBeInTheDocument();
  });

  it('renders underLabel under the title', () => {
    render(FeatureHeader, { label: 'Secret', underLabel: textSnippet('project: acme') });

    expect(screen.getByText('project: acme')).toBeInTheDocument();
  });
});
