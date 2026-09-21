import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { render } from '@/test/render.svelte.ts';
import MemberRowAction from './MemberRowAction.svelte';

const props = (overrides: Record<string, unknown> = {}) => ({
  isCurrentUser: false,
  canRemove: true,
  tooltip: 'Remove member',
  onRemove: vi.fn(),
  ...overrides,
});

describe('MemberRowAction', () => {
  it('marks your own row and offers no way to remove yourself', () => {
    render(MemberRowAction, props({ isCurrentUser: true }));

    // Self-removal would revoke the caller's own access mid-session, so the
    // control is absent rather than present-and-refused.
    expect(screen.getByText('You')).toBeInTheDocument();
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });

  it('offers removal on another member’s row, named by its tooltip', () => {
    render(MemberRowAction, props({ tooltip: 'Remove from project' }));

    expect(screen.getByRole('button', { name: 'Remove from project' })).toBeInTheDocument();
  });

  it('calls onRemove when pressed', async () => {
    const onRemove = vi.fn();
    render(MemberRowAction, props({ onRemove }));

    await userEvent.click(screen.getByRole('button', { name: 'Remove member' }));

    expect(onRemove).toHaveBeenCalled();
  });

  it('renders nothing at all when the caller may not remove this member', () => {
    render(MemberRowAction, props({ canRemove: false }));

    expect(screen.queryByRole('button')).not.toBeInTheDocument();
    expect(screen.queryByText('You')).not.toBeInTheDocument();
  });

  it('disables the control while a write is in flight', async () => {
    const onRemove = vi.fn();
    render(MemberRowAction, props({ disabled: true, onRemove }));

    const button = screen.getByRole('button', { name: 'Remove member' });
    expect(button).toBeDisabled();

    await userEvent.click(button);
    expect(onRemove).not.toHaveBeenCalled();
  });
});
