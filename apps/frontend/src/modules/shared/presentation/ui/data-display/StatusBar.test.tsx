import { describe, it, expect, vi } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import { renderWithI18n } from '@/test/render.tsx';
import userEvent from '@testing-library/user-event';
import { StatusBar } from './StatusBar';

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

  it('leaves a segment inert unless it was given an onSelect', () => {
    renderWithI18n(<StatusBar items={[{ id: 'a', status: 'completed' }]} />);
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });

  it('an activatable segment is a button named by its label, and does not bubble to the row', async () => {
    const onSelect = vi.fn();
    const rowClick = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(
      <div onClick={rowClick}>
        <StatusBar items={[{ id: 'a', status: 'completed', onSelect, label: 'Run #3' }]} />
      </div>,
    );

    await user.click(screen.getByRole('button', { name: 'Run #3' }));
    expect(onSelect).toHaveBeenCalled();
    expect(rowClick).not.toHaveBeenCalled();
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
