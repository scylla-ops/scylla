import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { fireEvent } from '@testing-library/react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { AgentIdLink } from './AgentIdLink';

const toastSuccess = vi.fn();
const toastError = vi.fn();
vi.mock('sonner', () => ({
  toast: {
    success: (...args: unknown[]) => toastSuccess(...args),
    error: (...args: unknown[]) => toastError(...args),
  },
}));

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

let writeText: ReturnType<typeof vi.fn>;

beforeEach(() => {
  toastSuccess.mockClear();
  toastError.mockClear();
  writeText = vi.fn().mockResolvedValue(undefined);
  Object.defineProperty(navigator, 'clipboard', {
    value: { writeText },
    configurable: true,
  });
});

describe('AgentIdLink', () => {
  it('shows the full id by default', () => {
    renderWithI18n(<AgentIdLink id='agent-abcdefgh-1234' />);
    expect(screen.getByText('agent-abcdefgh-1234')).toBeInTheDocument();
  });

  it('truncates when the id is longer than the given length', () => {
    renderWithI18n(<AgentIdLink id='agent-abcdefgh-1234' truncate={8} />);
    expect(screen.getByText('agent-ab…')).toBeInTheDocument();
  });

  it('does not truncate an id already shorter than the limit', () => {
    renderWithI18n(<AgentIdLink id='short-id' truncate={20} />);
    expect(screen.getByText('short-id')).toBeInTheDocument();
  });

  it('copies the full (untruncated) id and toasts on success', async () => {
    renderWithI18n(<AgentIdLink id='agent-abcdefgh-1234' truncate={8} />);
    fireEvent.click(screen.getByRole('button'));

    expect(writeText).toHaveBeenCalledWith('agent-abcdefgh-1234');
    await vi.waitFor(() => expect(toastSuccess).toHaveBeenCalledWith('Agent id copied'));
  });

  it('toasts an error when the clipboard write is denied, rather than failing silently', async () => {
    writeText.mockRejectedValueOnce(new Error('denied'));

    renderWithI18n(<AgentIdLink id='agent-1' />);
    fireEvent.click(screen.getByRole('button'));

    await vi.waitFor(() => expect(toastError).toHaveBeenCalledWith('Could not copy — select the id manually'));
    expect(toastSuccess).not.toHaveBeenCalled();
  });

  it('stops the click from bubbling up to a card\'s own onClick', () => {
    const cardOnClick = vi.fn();

    renderWithI18n(
      <div onClick={cardOnClick}>
        <AgentIdLink id='agent-1' />
      </div>,
    );
    fireEvent.click(screen.getByRole('button'));

    expect(cardOnClick).not.toHaveBeenCalled();
  });
});
