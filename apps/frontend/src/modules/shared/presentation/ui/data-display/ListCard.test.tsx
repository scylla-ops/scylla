import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ListCard } from './ListCard';

describe('ListCard', () => {
  it('renders every section\'s content', () => {
    render(
      <ListCard
        sections={[{ content: <span>Name</span> }, { content: <span>Status</span> }]}
      />,
    );
    expect(screen.getByText('Name')).toBeInTheDocument();
    expect(screen.getByText('Status')).toBeInTheDocument();
  });

  it('places a separator between sections, but never after the last one', () => {
    const { container } = render(
      <ListCard
        sections={[{ content: <span>A</span> }, { content: <span>B</span> }, { content: <span>C</span> }]}
      />,
    );
    // 3 sections -> at most 2 separators (never trailing the last section).
    expect(container.querySelectorAll('svg')).toHaveLength(2);
  });

  it('omits a separator when a section opts out with noSeparator', () => {
    const { container } = render(
      <ListCard
        sections={[
          { content: <span>A</span>, noSeparator: true },
          { content: <span>B</span> },
        ]}
      />,
    );
    expect(container.querySelectorAll('svg')).toHaveLength(0);
  });

  it('fires onClick when clicked', async () => {
    const onClick = vi.fn();
    const user = userEvent.setup();
    render(<ListCard sections={[{ content: <span>A</span> }]} onClick={onClick} />);
    await user.click(screen.getByText('A'));
    expect(onClick).toHaveBeenCalled();
  });

  it('a single section renders no separator at all', () => {
    const { container } = render(<ListCard sections={[{ content: <span>Only</span> }]} />);
    expect(container.querySelectorAll('svg')).toHaveLength(0);
  });
});
