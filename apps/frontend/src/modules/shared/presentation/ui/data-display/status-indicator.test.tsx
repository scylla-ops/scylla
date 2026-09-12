import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import StatusIndicator from './status-indicator';

describe('StatusIndicator', () => {
  it('renders the label when given one', () => {
    render(<StatusIndicator state='success' label='Completed' />);
    expect(screen.getByText('Completed')).toBeInTheDocument();
  });

  it('renders no label text when none is given', () => {
    const { container } = render(<StatusIndicator state='success' />);
    expect(container.querySelector('p')).toBeNull();
  });

  it('animates for a running state', () => {
    const { container } = render(<StatusIndicator state='running' />);
    expect(container.querySelector('.animate-ping')).not.toBeNull();
  });

  it('animates for a pending state', () => {
    const { container } = render(<StatusIndicator state='pending' />);
    expect(container.querySelector('.animate-ping')).not.toBeNull();
  });

  it('does not animate for a terminal state like success', () => {
    const { container } = render(<StatusIndicator state='success' />);
    expect(container.querySelector('.animate-ping')).toBeNull();
  });

  it('does not animate for a terminal state like failed', () => {
    const { container } = render(<StatusIndicator state='failed' />);
    expect(container.querySelector('.animate-ping')).toBeNull();
  });

  it('animateAllStates forces the ping animation even on a terminal state', () => {
    const { container } = render(<StatusIndicator state='failed' animateAllStates />);
    expect(container.querySelector('.animate-ping')).not.toBeNull();
  });

  it('cancelled and orphaned share the same color token', () => {
    const cancelled = render(<StatusIndicator state='cancelled' />);
    const cancelledDot = cancelled.container.querySelector('.bg-status-canceled');
    cancelled.unmount();

    const orphaned = render(<StatusIndicator state='orphaned' />);
    const orphanedDot = orphaned.container.querySelector('.bg-status-canceled');

    expect(cancelledDot).not.toBeNull();
    expect(orphanedDot).not.toBeNull();
  });

  it('falls back to the idle/muted styling for an unrecognized state at runtime', () => {
    // The type is a closed union, but the value can still come from an API at runtime.
    const { container } = render(
      <StatusIndicator state={'something-new' as unknown as 'idle'} />,
    );
    expect(container.querySelector('.bg-muted-foreground\\/60')).not.toBeNull();
  });
});
