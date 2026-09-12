import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { act, render, screen, fireEvent } from '@testing-library/react';
import { CopyableText } from './CopyableText';

let writeText: ReturnType<typeof vi.fn>;

beforeEach(() => {
  writeText = vi.fn().mockResolvedValue(undefined);
  Object.defineProperty(navigator, 'clipboard', {
    value: { writeText },
    configurable: true,
  });
});

afterEach(() => {
  vi.useRealTimers();
});

describe('CopyableText', () => {
  it('renders the full value by default', () => {
    render(<CopyableText value='job-01m27ybd6ym31g3h883mk86bse' />);
    expect(screen.getByText('job-01m27ybd6ym31g3h883mk86bse')).toBeInTheDocument();
  });

  it('truncates the displayed text and adds an ellipsis, without truncating the copied value', () => {
    render(<CopyableText value='job-01m27ybd6ym31g3h883mk86bse' truncate={8} />);
    expect(screen.getByText('job-01m2...')).toBeInTheDocument();
  });

  it('renders a custom display node instead of the value when given one', () => {
    render(<CopyableText value='secret-value' display='••••••••' />);
    expect(screen.getByText('••••••••')).toBeInTheDocument();
    expect(screen.queryByText('secret-value')).not.toBeInTheDocument();
  });

  it('copies the full (untruncated) value to the clipboard on click', () => {
    render(<CopyableText value='the-full-value-1234567890' truncate={4} />);
    fireEvent.click(screen.getByRole('button'));
    expect(writeText).toHaveBeenCalledWith('the-full-value-1234567890');
  });

  it('does not let the click bubble up to a parent (e.g. a clickable table row)', () => {
    const onRowClick = vi.fn();
    render(
      <div onClick={onRowClick}>
        <CopyableText value='row-id' />
      </div>,
    );
    fireEvent.click(screen.getByRole('button'));
    expect(onRowClick).not.toHaveBeenCalled();
  });

  it('shows a checkmark state after copying, then reverts after 2s', () => {
    vi.useFakeTimers();
    const { container } = render(<CopyableText value='x' />);

    fireEvent.click(screen.getByRole('button'));
    expect(container.querySelector('.text-status-passed')).not.toBeNull();

    act(() => {
      vi.advanceTimersByTime(2000);
    });
    expect(container.querySelector('.text-status-passed')).toBeNull();
  });
});
