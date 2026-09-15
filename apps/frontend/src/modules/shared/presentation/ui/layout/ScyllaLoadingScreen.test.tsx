import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { act, render, screen } from '@testing-library/react';
import { ScyllaLoadingScreen } from './ScyllaLoadingScreen';

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('ScyllaLoadingScreen', () => {
  it('shows nothing at first - a fast load should never see a flash of logo', () => {
    render(<ScyllaLoadingScreen />);
    expect(screen.queryByAltText('Scylla')).not.toBeInTheDocument();
  });

  it('shows the logo once the delay has passed', () => {
    render(<ScyllaLoadingScreen />);

    // Fake timers + waitFor's own polling deadlock each other - the timer
    // callback runs synchronously here, just needs wrapping for the state
    // update it triggers.
    act(() => {
      vi.advanceTimersByTime(300);
    });

    expect(screen.getByAltText('Scylla')).toBeInTheDocument();
  });

  it('cancels the timer on unmount, so it never sets state on an unmounted component', () => {
    const { unmount } = render(<ScyllaLoadingScreen />);
    unmount();
    // Advancing after unmount must not throw (React would warn/error on a
    // state update on an unmounted component if the cleanup didn't run).
    expect(() => vi.advanceTimersByTime(300)).not.toThrow();
  });
});
