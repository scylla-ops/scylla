import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { FeatureHeader } from './FeatureHeader';

const toastSuccess = vi.fn();
vi.mock('sonner', () => ({
  toast: { success: (...args: unknown[]) => toastSuccess(...args) },
}));

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

beforeEach(() => {
  toastSuccess.mockClear();
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
});

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

// The delete button is icon-only (a bare Trash icon, aria-hidden) - it carries
// no accessible name, so it can't be found by role+name like the others.
const getDeleteButton = (): HTMLElement => {
  const button = document.querySelector('button[data-variant="destructive"]');
  if (!button) throw new Error('delete button not found');
  return button as HTMLElement;
};

describe('FeatureHeader', () => {
  it('shows the count and singular label', () => {
    renderWithI18n(<FeatureHeader count={1} label='pipeline' />);
    expect(screen.getByText('1')).toBeInTheDocument();
    expect(screen.getByText('pipeline')).toBeInTheDocument();
  });

  it('switches to the plural label once count > 1', () => {
    renderWithI18n(<FeatureHeader count={5} label='pipeline' pluralLabel='pipelines' />);
    expect(screen.getByText('pipelines')).toBeInTheDocument();
    expect(screen.queryByText('pipeline')).not.toBeInTheDocument();
  });

  it('falls back to the singular label when no plural was given, even with count > 1', () => {
    renderWithI18n(<FeatureHeader count={5} label='pipeline' />);
    expect(screen.getByText('pipeline')).toBeInTheDocument();
  });

  it('omits "Select all" once everything is already selected', () => {
    renderWithI18n(
      <FeatureHeader count={3} label='item' onSelectAll={vi.fn()} allSelected />,
    );
    expect(screen.queryByRole('button', { name: 'Select all' })).not.toBeInTheDocument();
  });

  it('shows "Clear" only once something is selected', () => {
    const { rerender } = renderWithI18n(
      <FeatureHeader count={3} label='item' selectedCount={0} onClearSelection={vi.fn()} />,
    );
    expect(screen.queryByRole('button', { name: 'Clear' })).not.toBeInTheDocument();

    rerender(
      <I18nProvider i18n={i18n}>
        <FeatureHeader count={3} label='item' selectedCount={2} onClearSelection={vi.fn()} />
      </I18nProvider>,
    );
    expect(screen.getByRole('button', { name: 'Clear' })).toBeInTheDocument();
  });

  it('calls onNew when New is clicked', async () => {
    const onNew = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<FeatureHeader label='pipeline' onNew={onNew} />);
    await user.click(screen.getByRole('button', { name: 'New pipeline' }));
    expect(onNew).toHaveBeenCalled();
  });

  it('disables New with a tooltip when canNew is false', async () => {
    const onNew = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<FeatureHeader label='pipeline' onNew={onNew} canNew={false} />);

    const button = screen.getByRole('button', { name: 'New pipeline' });
    expect(button).toBeDisabled();

    await user.hover(button);
    await waitFor(() => expect(screen.getByRole('tooltip')).toHaveTextContent("don't have permission"));
  });

  it('opens a confirmation dialog before deleting, then toasts on success', async () => {
    const onDeleteSelection = vi.fn().mockResolvedValue(undefined);
    const user = userEvent.setup();
    renderWithI18n(
      <FeatureHeader
        label='pipeline'
        selectedCount={2}
        onDeleteSelection={onDeleteSelection}
      />,
    );

    await user.click(getDeleteButton());
    expect(onDeleteSelection).not.toHaveBeenCalled();
    expect(screen.getByText('Are you absolutely sure?')).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Continue' }));

    await waitFor(() => expect(onDeleteSelection).toHaveBeenCalled());
    await waitFor(() => expect(toastSuccess).toHaveBeenCalledWith('2 items deleted'));
  });

  it('singularizes the delete-success toast for exactly one item', async () => {
    const onDeleteSelection = vi.fn().mockResolvedValue(undefined);
    const user = userEvent.setup();
    renderWithI18n(
      <FeatureHeader label='pipeline' selectedCount={1} onDeleteSelection={onDeleteSelection} />,
    );

    await user.click(getDeleteButton());
    await user.click(screen.getByRole('button', { name: 'Continue' }));

    await waitFor(() => expect(toastSuccess).toHaveBeenCalledWith('1 item deleted'));
  });

  it('disables the delete action with a tooltip when canDelete is false', async () => {
    const user = userEvent.setup();
    renderWithI18n(
      <FeatureHeader
        label='pipeline'
        selectedCount={2}
        onDeleteSelection={vi.fn()}
        canDelete={false}
      />,
    );

    const button = getDeleteButton();
    expect(button).toBeDisabled();

    await user.hover(button);
    await waitFor(() => expect(screen.getByRole('tooltip')).toHaveTextContent("don't have permission"));
  });

  it('does not toast when the delete call fails, and closes the dialog anyway', async () => {
    const onDeleteSelection = vi.fn().mockRejectedValue(new Error('denied'));
    const user = userEvent.setup();
    renderWithI18n(
      <FeatureHeader label='pipeline' selectedCount={1} onDeleteSelection={onDeleteSelection} />,
    );

    await user.click(getDeleteButton());
    await user.click(screen.getByRole('button', { name: 'Continue' }));

    await waitFor(() => expect(onDeleteSelection).toHaveBeenCalled());
    expect(toastSuccess).not.toHaveBeenCalled();
    await waitFor(() => expect(screen.queryByText('Are you absolutely sure?')).not.toBeInTheDocument());
  });

  it('renders extraActions alongside the built-in ones', () => {
    renderWithI18n(<FeatureHeader label='pipeline' extraActions={<button>Export</button>} />);
    expect(screen.getByRole('button', { name: 'Export' })).toBeInTheDocument();
  });
});
