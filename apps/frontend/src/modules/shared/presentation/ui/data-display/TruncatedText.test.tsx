import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { TruncatedText } from './TruncatedText';

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

beforeEach(() => {
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
});

/** jsdom never lays out text, so scrollWidth/clientWidth are always 0 - fake
 * whichever relationship the test needs on the rendered span. */
const setOverflow = (element: HTMLElement, { truncated }: { truncated: boolean }) => {
  Object.defineProperty(element, 'scrollWidth', { configurable: true, value: truncated ? 200 : 100 });
  Object.defineProperty(element, 'clientWidth', { configurable: true, value: 100 });
};

describe('TruncatedText', () => {
  it('never shows a tooltip before the text has been measured (no pointer/focus event yet)', () => {
    render(<TruncatedText>A long line of text</TruncatedText>);
    expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
  });

  it('reveals a tooltip once measured as actually overflowing', async () => {
    const user = userEvent.setup();
    render(<TruncatedText>A long line of text</TruncatedText>);

    const span = screen.getByText('A long line of text');
    setOverflow(span, { truncated: true });
    await user.hover(span);

    await waitFor(() => expect(screen.getByRole('tooltip')).toHaveTextContent('A long line of text'));
  });

  it('never shows a tooltip when the text fits - repeating visible text would be noise', async () => {
    const user = userEvent.setup();
    render(<TruncatedText>Short</TruncatedText>);

    const span = screen.getByText('Short');
    setOverflow(span, { truncated: false });
    await user.hover(span);

    expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
  });

  it('shows a custom tooltip body instead of repeating the children', async () => {
    const user = userEvent.setup();
    render(<TruncatedText tooltip='Full untruncated value'>short label</TruncatedText>);

    const span = screen.getByText('short label');
    setOverflow(span, { truncated: true });
    await user.hover(span);

    await waitFor(() => expect(screen.getByRole('tooltip')).toHaveTextContent('Full untruncated value'));
  });
});
