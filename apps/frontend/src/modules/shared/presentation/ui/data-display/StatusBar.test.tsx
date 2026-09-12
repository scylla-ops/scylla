import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { StatusBar } from './StatusBar';

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

beforeEach(() => {
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
});

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

describe('StatusBar', () => {
  it('shows "No data" (default) when there are no items', () => {
    renderWithI18n(<StatusBar items={[]} />);
    expect(screen.getByText('No data')).toBeInTheDocument();
  });

  it('shows a custom empty label instead', () => {
    renderWithI18n(<StatusBar items={[]} emptyLabel='Nothing to show yet' />);
    expect(screen.getByText('Nothing to show yet')).toBeInTheDocument();
  });

  it('renders one segment per item', () => {
    const { container } = renderWithI18n(
      <StatusBar items={[{ id: 'a', status: 'completed' }, { id: 'b', status: 'failed' }]} />,
    );
    expect(container.querySelectorAll('.flex-1')).toHaveLength(2);
  });

  it('only wraps a segment with a tooltip when it has one', async () => {
    const user = userEvent.setup();
    renderWithI18n(
      <StatusBar
        items={[
          { id: 'a', status: 'completed', tooltip: 'job A' },
          { id: 'b', status: 'failed' },
        ]}
      />,
    );

    expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();

    const [withTooltip] = document.querySelectorAll('.flex-1');
    await user.hover(withTooltip);
    await waitFor(() => expect(screen.getByRole('tooltip')).toHaveTextContent('job A'));
  });
});
